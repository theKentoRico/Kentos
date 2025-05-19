# Shell

Kentos has a shell whih aims to be bash-like except that it uses '>' instead of '$'.

## Keyword commands

I like to refer these as such because they are commands but they're NOT executable files and are implemented like keywords in other programming languages.

They are:

### 1. greet

`greet` takes no arguments and just prints `Hello`.

### 2. echo

`echo` takes as many arguments as you want and prints it e.g.

`echo rust is great` outputs:

`rust is great`

### 3. reboot

`reboot` takes no arguments and reboots the system by re-executing the `k_start` function from the beginning.