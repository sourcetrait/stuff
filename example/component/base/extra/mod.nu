const ID: string = 'base/extra'

def marker []: nothing -> path {
    $env.HOME | path join .local state stuff-example ($ID | str replace --all '/' '-')
}

export def present []: nothing -> bool {
    marker | path exists
}

export def add []: nothing -> nothing {
    mkdir (marker | path dirname)
    for step in 1..5 {
        print $'($ID): step ($step) of 5'
        sleep 300ms
    }
    date now | format date '%+' | save --force (marker)
}

export def remove []: nothing -> nothing {
    print $'($ID): removing'
    rm --force (marker)
}
