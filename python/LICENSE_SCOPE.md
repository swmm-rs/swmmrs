# Python distribution license scope

Original Python interface and native-binding source in this directory is
licensed under the Apache License, Version 2.0, reproduced in [`LICENSE`](LICENSE).

The native solver is excluded from that grant. A wheel combines the
Apache-2.0-licensed interface with compiled solver object code. Official wheels
include the swmmrs Binary Runtime License, Version 1.0, in
[`licenses/SWMMRS-BINARY-RUNTIME.txt`](licenses/SWMMRS-BINARY-RUNTIME.txt). It
permits use, modification, and redistribution of official compiled release
artifacts for any purpose, but it does not grant access to undistributed private
solver source.

Bundled upstream material remains under its own terms. See
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) and the files under
`licenses/`.
