# Writes this example component's marker, slowly enough to watch.
def main [] {
    let name = ($env.COMPONENT_ID | str replace --all '/' '-')
    let dir = ($env.HOME | path join .local state stuff-example)
    mkdir $dir
    for step in 1..5 {
        print $'($env.COMPONENT_ID): step ($step) of 5'
        sleep 300ms
    }
    date now | format date '%+' | save --force ($dir | path join $name)
}
