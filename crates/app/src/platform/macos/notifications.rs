//! The notifications on macOS (`MOD-11-F10`, `DEC-218`): from the app's bundle, through the
//! system's notification center in the app's own name, with Join, Snooze and Dismiss on a
//! reminder, Accept, Maybe and Decline on an invitation (`MOD-07-F23`), and a click that opens the
//! event. A program run outside the bundle, as `cargo run` makes, has no name
//! the notification center knows: its notifications go through AppleScript's `display
//! notification`, without buttons and saying nothing back (`DEC-64`).

use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, Once};

use calendar_core::platform::Notifier;
use calendar_core::reminders::{Action, Notification};
use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{AllocAnyThread, define_class, msg_send};
use objc2_foundation::{NSArray, NSBundle, NSError, NSSet, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotification, UNNotificationAction,
    UNNotificationActionOptions, UNNotificationCategory, UNNotificationCategoryOptions,
    UNNotificationDefaultActionIdentifier, UNNotificationPresentationOptions,
    UNNotificationRequest, UNNotificationResponse, UNNotificationSound, UNUserNotificationCenter,
    UNUserNotificationCenterDelegate,
};

use super::super::{action, key};

type Chosen = Box<dyn Fn(Action) + Send + Sync>;

/// What each notification shown calls with what the user chooses on it, by its identifier.
static CHOSEN: LazyLock<Mutex<HashMap<String, Chosen>>> = LazyLock::new(Mutex::default);

/// The category of a notification with `actions`, which gives it their buttons.
fn category(actions: &[Action]) -> String {
    actions
        .iter()
        .map(|action| key(*action))
        .collect::<Vec<_>>()
        .join("+")
}

/// What the user chose: clicking the notification opens it, a button does its action.
fn chosen(pressed: &str) -> Option<Action> {
    let clicked = unsafe { UNNotificationDefaultActionIdentifier };
    if pressed == clicked.to_string() {
        Some(Action::Open)
    } else {
        action(pressed)
    }
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and `Delegate` does not implement
    // `Drop`.
    #[unsafe(super(NSObject))]
    /// Hears from the notification center what the user chose on a notification.
    struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl UNUserNotificationCenterDelegate for Delegate {
        /// A notification is shown also while the app is in front.
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn will_present(
            &self,
            _center: &UNUserNotificationCenter,
            _notification: &UNNotification,
            completion_handler: &block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            completion_handler.call((UNNotificationPresentationOptions::Banner
                | UNNotificationPresentationOptions::List
                | UNNotificationPresentationOptions::Sound,));
        }

        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        fn did_receive(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            completion_handler: &block2::DynBlock<dyn Fn()>,
        ) {
            let id = response.notification().request().identifier().to_string();
            let pressed = response.actionIdentifier().to_string();
            let chosen_by = CHOSEN.lock().expect("chosen").remove(&id);
            if let (Some(action), Some(chosen_by)) = (chosen(&pressed), chosen_by) {
                chosen_by(action);
            }
            completion_handler.call(());
        }
    }
);

impl Delegate {
    fn new() -> Retained<Self> {
        let this = Self::alloc().set_ivars(());
        unsafe { msg_send![super(this), init] }
    }
}

/// Hands the notification center its delegate and the buttons of each kind of notification,
/// and asks the user, the first time, to let the app show notifications.
fn set_up() {
    let center = UNUserNotificationCenter::currentNotificationCenter();
    let delegate = Delegate::new();
    center.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    // The notification center does not hold its delegate.
    std::mem::forget(delegate);
    let categories: Vec<Retained<UNNotificationCategory>> = Action::button_sets()
        .map(|actions| {
            let buttons: Vec<Retained<UNNotificationAction>> = actions
                .iter()
                .map(|action| {
                    UNNotificationAction::actionWithIdentifier_title_options(
                        &NSString::from_str(&key(*action)),
                        &NSString::from_str(&action.label()),
                        UNNotificationActionOptions::empty(),
                    )
                })
                .collect();
            UNNotificationCategory::categoryWithIdentifier_actions_intentIdentifiers_options(
                &NSString::from_str(&category(&actions)),
                &NSArray::from_retained_slice(&buttons),
                &NSArray::new(),
                UNNotificationCategoryOptions::empty(),
            )
        })
        .into();
    center.setNotificationCategories(&NSSet::from_retained_slice(&categories));
    let answered = block2::RcBlock::new(|_: objc2::runtime::Bool, _: *mut NSError| {});
    center.requestAuthorizationWithOptions_completionHandler(
        UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
        &answered,
    );
}

/// The notifications, through the notification center from the bundle and through AppleScript
/// outside it.
pub struct SystemNotifier {
    /// Whether the program runs from the app's bundle.
    bundled: bool,
}

impl SystemNotifier {
    pub fn new() -> Self {
        // Outside a bundle the notification center ends the program.
        let bundled = NSBundle::mainBundle().bundleIdentifier().is_some();
        if bundled {
            static SET_UP: Once = Once::new();
            SET_UP.call_once(set_up);
        }
        SystemNotifier { bundled }
    }

    /// Shows the notification through the notification center.
    fn post(&self, notification: &Notification, chosen_by: Chosen) {
        static SHOWN: AtomicU64 = AtomicU64::new(0);
        let id = format!(
            "{}-{}",
            std::process::id(),
            SHOWN.fetch_add(1, Ordering::Relaxed)
        );
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(&notification.title));
        content.setBody(&NSString::from_str(&notification.body()));
        content.setCategoryIdentifier(&NSString::from_str(&category(&notification.actions())));
        content.setSound(Some(&UNNotificationSound::defaultSound()));
        let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
            &NSString::from_str(&id),
            &content,
            None,
        );
        CHOSEN.lock().expect("chosen").insert(id, chosen_by);
        UNUserNotificationCenter::currentNotificationCenter()
            .addNotificationRequest_withCompletionHandler(&request, None);
    }
}

impl Notifier for SystemNotifier {
    fn show(&self, notification: &Notification, chosen_by: Chosen) {
        if self.bundled {
            return self.post(notification, chosen_by);
        }
        // The texts go as the script's arguments, so nothing in them is read as AppleScript.
        let shown = Command::new("osascript")
            .args([
                "-e",
                "on run argv",
                "-e",
                "display notification (item 2 of argv) with title (item 1 of argv)",
                "-e",
                "end run",
                notification.title.as_str(),
                &notification.body(),
            ])
            .spawn();
        if let Ok(mut child) = shown {
            std::thread::spawn(move || child.wait());
        }
    }

    fn buttons(&self) -> bool {
        self.bundled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notification_with_google_meet_has_its_own_buttons() {
        assert_eq!(
            category(&Action::buttons(false)),
            "snooze-5+snooze-10+dismiss"
        );
        assert_eq!(
            category(&Action::buttons(true)),
            "join+snooze-5+snooze-10+dismiss"
        );
        assert_eq!(category(&Action::answers()), "accept+maybe+decline");
    }

    #[test]
    fn a_click_opens_the_notification_and_a_button_does_its_action() {
        let clicked = unsafe { UNNotificationDefaultActionIdentifier }.to_string();
        assert_eq!(chosen(&clicked), Some(Action::Open));
        for action in Action::buttons(true) {
            assert_eq!(chosen(&key(action)), Some(action));
        }
        assert_eq!(
            chosen("com.apple.UNNotificationDismissActionIdentifier"),
            None
        );
    }
}
