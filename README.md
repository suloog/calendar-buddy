# The delivery loop's state

This branch is written by `og-delivery backup` (the og plugin's delivery loop): `state/` is the
live state of a run — its settings, event log, implementer results and plans, DEC register and
cost records — saved after every iteration, one commit on top of the last. A clone that lost its
state is rebuilt from here (`og-delivery backup --restore`).

Nobody works on this branch. It is never merged and opens no pull request, and it can be deleted
once the run's report is on the base.
