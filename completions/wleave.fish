complete -c wleave -s l -l layout -d 'Specify a layout file' -r -F
complete -c wleave -s C -l css -d 'Specify a custom CSS file' -r -F
complete -c wleave -s b -l buttons-per-row -d 'Set the number of buttons per row' -r
complete -c wleave -s c -l column-spacing -d 'Set space between buttons columns' -r
complete -c wleave -s r -l row-spacing -d 'Set space between buttons rows' -r
complete -c wleave -s m -l margin -d 'Set the margin around buttons' -r
complete -c wleave -s L -l margin-left -d 'Set margin for the left of buttons' -r
complete -c wleave -s R -l margin-right -d 'Set margin for the right of buttons' -r
complete -c wleave -s T -l margin-top -d 'Set margin for the top of buttons' -r
complete -c wleave -s B -l margin-bottom -d 'Set the margin for the bottom of buttons' -r
complete -c wleave -s p -l protocol -d 'Use layer-shell or xdg protocol' -r -f -a "{layer-shell	,xdg	}"
complete -c wleave -s v -l version
complete -c wleave -s f -l close-on-lost-focus -d 'Close the menu on lost focus'
complete -c wleave -s k -l show-keybinds -d 'Show the associated key binds'
complete -c wleave -s h -l help -d 'Print help'
