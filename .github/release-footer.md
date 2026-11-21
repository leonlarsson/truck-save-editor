
### Download

Download **truck-save-editor-{version}-windows-x64.zip** below, unzip it and run `truck-save-editor.exe`. It runs on Windows 10 and 11 with nothing to install.

Windows may show "Windows protected your PC" at first, because the app isn't code-signed yet. Choose **More info**, then **Run anyway**.

Close the game before saving: it overwrites saves when it exits. Every save is backed up before it's written, and Backups puts one back.

### Check the download

`SHA256SUMS.txt` has the SHA-256 checksum of the zip and of the .exe inside it. With the [GitHub CLI](https://cli.github.com/) you can also check that a file was built from this repository by its release workflow:

```
gh attestation verify truck-save-editor-{version}-windows-x64.zip --repo {repository}
```
