# Windows installer release checklist

Complete this checklist on a clean, fully updated Windows 11 x64 virtual machine using
artifacts downloaded from the draft GitHub Release. Record the tag, commit, Windows
build, asset names, SHA-256 values and tester.

## Asset verification

- [ ] The draft contains `QRY_<version>_x64-setup.exe`, `QRY_<version>_x64_en-US.msi`
      and one `.sha256` file for each.
- [ ] Locally calculated SHA-256 values match both checksum files.
- [ ] The installer contains no SQLite database, CSV export, `.env`, log or development
      fixture.
- [ ] If Authenticode signing is required for this release, both installers report a
      valid signature from the approved publisher. No certificate or password is stored
      in the repository.

## NSIS current-user installation

- [ ] Launch the setup executable as a standard user and confirm it does not request
      administrator privileges.
- [ ] If SmartScreen warns about an unsigned build, confirm the filename and checksum
      before using the scoped **More info → Run anyway** action. Never disable
      SmartScreen globally.
- [ ] Complete installation and confirm QRY launches, shows empty statistics, completes
      onboarding and monitors input in real applications.
- [ ] Confirm WebView2 is reused when present and the download bootstrapper is offered
      only when required.
- [ ] Enable launch at login, sign out/in, then disable it and repeat.
- [ ] Install the same version again and confirm there is no duplicate startup
      registration or second data directory.
- [ ] Uninstall QRY and confirm the application and startup registration are removed.
      Record separately whether user-created aggregate data remains.

## MSI installation

- [ ] Install the MSI with the standard Windows Installer UI and repeat the clean
      first-run and empty-statistics checks.
- [ ] Repair or reinstall once and confirm the existing aggregate database is readable
      and no demo data is introduced.
- [ ] Uninstall with Windows Settings and confirm no executable, tray process or startup
      registration remains.

Do not publish the draft until this checklist and the Windows input, shell and lifecycle
checklists are signed off. Any signing, install, uninstall, privacy or data-loss failure
blocks publication.
