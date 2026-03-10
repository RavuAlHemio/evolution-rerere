# evolution-rerere

A module for [GNOME Evolution](https://gitlab.gnome.org/GNOME/evolution) that
reduces multiple `Fwd:` or `Re:` prefixes in subjects to the first one.

Also supports formats with a depth counter, incrementing `Re[1]:` to `Re[2]:`.

Prefix configurability is not yet available; the prefixes being reduced are
those according to Unix tradition (`Re:` and `Fwd:`), according to Outlook in
English (`RE:` and `FW:`), and according to Outlook in German (`AW:` and
`WG:`).
