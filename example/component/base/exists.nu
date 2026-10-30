# Exits 0 when this example component's marker exists.
def main [] {
    let name = ($env.COMPONENT_ID | str replace --all '/' '-')
    let marker = ($env.HOME | path join .local state stuff-example $name)
    if ($marker | path exists) { exit 0 } else { exit 1 }
}
