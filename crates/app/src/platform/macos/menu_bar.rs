//! The app's icon in the menu bar (`MOD-11-F09`, `DEC-218`): a click opens the window, and a
//! right click shows a menu that opens the window and quits the app (`MOD-11-U03`, `DEC-220`).
//! While the window is hidden the app is not in the Dock, and opening it again from Spotlight,
//! Launchpad or Finder brings the window back.

use std::ptr::NonNull;
use std::sync::Arc;

use calendar_core::background::Signal;
use calendar_core::platform::Tray;
use calendar_core::texts;
use dispatch2::DispatchQueue;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{
    AllocAnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDidBecomeActiveNotification,
    NSEventMask, NSEventModifierFlags, NSEventType, NSImage, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSData, NSNotification, NSNotificationCenter, NSOperationQueue, NSSize, NSString,
};

use super::super::ICON;

type Chosen = Arc<dyn Fn(Signal) + Send + Sync>;

/// What the icon and its menu call, with the icon and the menu it pops up.
struct Targets {
    chosen: Chosen,
    item: Retained<NSStatusItem>,
    menu: Retained<NSMenu>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and `MenuTarget` does not implement
    // `Drop`.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Targets]
    /// What the icon and the menu's items call.
    struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(clicked:))]
        fn clicked(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let event = NSApplication::sharedApplication(mtm).currentEvent();
            let wants_menu = event.is_some_and(|event| {
                event.r#type() == NSEventType::RightMouseUp
                    || event.modifierFlags().contains(NSEventModifierFlags::Control)
            });
            let targets = self.ivars();
            if wants_menu {
                // The menu set on the icon would open on every click.
                #[allow(deprecated)]
                targets.item.popUpStatusItemMenu(&targets.menu);
            } else {
                (targets.chosen)(Signal::Open);
            }
        }

        #[unsafe(method(open:))]
        fn open(&self, _sender: Option<&AnyObject>) {
            (self.ivars().chosen)(Signal::Open);
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&AnyObject>) {
            (self.ivars().chosen)(Signal::Quit);
        }
    }
);

impl MenuTarget {
    fn new(mtm: MainThreadMarker, targets: Targets) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(targets);
        unsafe { msg_send![super(this), init] }
    }
}

/// The icon: the system's calendar symbol, which macOS draws in the menu bar's colours; the
/// app's own icon if the symbol is missing.
fn icon() -> Option<Retained<NSImage>> {
    let symbol = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str("calendar"),
        Some(&NSString::from_str(texts::APP_NAME)),
    );
    symbol.or_else(|| {
        let image = NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(ICON))?;
        image.setSize(NSSize::new(18.0, 18.0));
        Some(image)
    })
}

/// Puts the icon with its menu in the menu bar, for as long as the app runs: a click opens the
/// window, a right or Control click shows the menu (`DEC-220`).
fn status_item(mtm: MainThreadMarker, chosen: Chosen) {
    let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
    let menu = NSMenu::new(mtm);
    let target = MenuTarget::new(
        mtm,
        Targets {
            chosen,
            item: item.clone(),
            menu: menu.clone(),
        },
    );
    if let Some(button) = item.button(mtm) {
        button.setImage(icon().as_deref());
        unsafe {
            button.setTarget(Some(&target));
            button.setAction(Some(sel!(clicked:)));
        }
        button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
    }
    let entry = |title: &str, action| {
        let entry = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                Some(action),
                &NSString::new(),
            )
        };
        unsafe { entry.setTarget(Some(&target)) };
        entry
    };
    menu.addItem(&entry(texts::OPEN_APP, sel!(open:)));
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    menu.addItem(&entry(texts::QUIT, sel!(quit:)));
    // Neither the status bar nor the icon's button retains the target: leaking it keeps it, the
    // icon and the menu for as long as the app runs.
    std::mem::forget(target);
}

/// Opening the app while it runs, from Spotlight, Launchpad, Finder or a clicked notification,
/// makes it active: the window is brought back as on a second launch (`MOD-07-F17`).
fn watch_activation(chosen: Chosen) {
    let block = block2::RcBlock::new(move |_: NonNull<NSNotification>| chosen(Signal::Launched));
    let observer = unsafe {
        NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
            Some(NSApplicationDidBecomeActiveNotification),
            None,
            Some(&NSOperationQueue::mainQueue()),
            &block,
        )
    };
    std::mem::forget(observer);
}

/// The icon in the menu bar. It is put there from the main queue, which runs once the window's
/// event loop does: AppKit's application has to be made by that loop first.
pub struct SystemTray;

impl SystemTray {
    pub fn new() -> Self {
        SystemTray
    }
}

impl Tray for SystemTray {
    fn show(&self, chosen: Box<dyn Fn(Signal) + Send + Sync>, _at_login: bool) -> bool {
        let chosen: Chosen = Arc::from(chosen);
        DispatchQueue::main().exec_async(move || {
            let mtm = MainThreadMarker::new().expect("the main queue runs on the main thread");
            status_item(mtm, chosen.clone());
            watch_activation(chosen);
        });
        true
    }
}

/// Puts the app in the Dock and in front while its window is shown, and takes it out of the
/// Dock and away from the front while the window is hidden (`DEC-218`). Called on the main
/// thread, once the window's event loop runs.
pub fn in_dock(shown: bool) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    if shown {
        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        app.unhide(None);
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
    } else {
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        app.hide(None);
    }
}
