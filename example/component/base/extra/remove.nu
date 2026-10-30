# Removes this example component's marker.
def main [] {
    let name = ($env.COMPONENT_ID | str replace --all '/' '-')
    print $'($env.COMPONENT_ID): removing'
    rm --force ($env.HOME | path join .local state stuff-example $name)
}
