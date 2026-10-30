# error.rs

## enum Error

The crate's error wraps each module's transparently, so its message is the
module error's own: a command's stderr reads `stuff: <the failure>` with no
layer of prefixes.

## enum RunnerError

### Script

Names the component and the script as `<component> <script>`, `base/extra
add.nu`, the pair a person needs to find the failing file.
