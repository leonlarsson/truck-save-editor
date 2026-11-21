# Third-party notices

Code in this repository adapted from other projects, with their licenses.

## scs_tools

`crates/hashfs/src/gdeflate.rs` (GDeflate decompression) and `crates/hashfs/src/cityhash.rs` (CityHash64 as SCS uses it) are ported from [scs_tools](https://github.com/Wilps93/scs_tools). The HashFS reader in `crates/hashfs` follows its `FORMAT.md`.

```text
MIT License

Copyright (c) 2026 scs_tools contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

CityHash itself is Copyright (c) 2011 Google, Inc., under the MIT license. GDeflate decompression derives from NVIDIA's libdeflate (gdeflate branch), MIT license.
