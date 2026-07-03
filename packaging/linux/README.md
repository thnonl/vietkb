# Linux packaging

Targets:

- `deb/`
- `rpm/`
- `arch/`

IME integration packages must install engine metadata, DBus service files, icons, and restart instructions for the selected input framework.

Default behavior:

- IBus/Fcitx5 direct commit.
- Surrounding text delete before replacement commit.
- Preedit only as fallback.
