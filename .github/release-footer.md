
### Download

Download **truck-save-editor-{version}-windows-x64.exe** below and run it. It runs on Windows 10 and 11 with nothing to install; put it wherever you like.

Windows may show "Windows protected your PC" at first, because the app isn't code-signed yet. Choose **More info**, then **Run anyway**.

Close the game before saving: it overwrites saves when it exits. Every save is backed up before it's written, and Backups puts one back.

### Check the download

`SHA256SUMS.txt` has the .exe's SHA-256 checksum. With the [GitHub CLI](https://cli.github.com/) you can also check that a file was built from this repository by its release workflow:

```
gh attestation verify truck-save-editor-{version}-windows-x64.exe --repo {repository}
```
