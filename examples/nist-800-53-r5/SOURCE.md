# Source of vendored NIST SP 800-53 Rev 5 OSCAL content

These files are unmodified copies of official NIST OSCAL content.

- Upstream repository: https://github.com/usnistgov/oscal-content
- Commit: `78650f02ad9321bb7b817846f8fbd4f2bcd620de` (2026-05-13T05:13:20Z, "Publishing auto-converted artifacts [skip ci]")

| Local file | Upstream path | Bytes | Git blob SHA-1 | SHA-256 |
| --- | --- | --- | --- | --- |
| `NIST_SP-800-53_rev5_catalog.json` | `nist.gov/SP800-53/rev5/json/NIST_SP-800-53_rev5_catalog.json` | 10442037 | `8ebe3c91181cc078d40e39bcf34ea233e652d025` | `01f37cf90ea99d92242c936cbfbdebcc338eef1f71454e2acac36cc56e9bc062` |
| `NIST_SP-800-53_rev5_MODERATE-baseline_profile.json` | `nist.gov/SP800-53/rev5/json/NIST_SP-800-53_rev5_MODERATE-baseline_profile.json` | 10498 | `a5da8cff72abe52b819b69dbe225ad7c4d780c8e` | `9030dbf1f13169947eb97eb101b4bd2f00d3c151b100455a923ac75803f00ea1` |

## Verification

The git blob SHA-1 of each local file (`git hash-object <file>`) equals the blob
SHA-1 GitHub reports for the upstream path at the commit above, which shows the
local copies are byte-identical to upstream. To re-check:

```sh
git hash-object examples/nist-800-53-r5/NIST_SP-800-53_rev5_catalog.json
gh api "repos/usnistgov/oscal-content/contents/nist.gov/SP800-53/rev5/json/NIST_SP-800-53_rev5_catalog.json?ref=78650f02ad9321bb7b817846f8fbd4f2bcd620de" --jq .sha
shasum -a 256 examples/nist-800-53-r5/*.json
```

## Known upstream content issue

The catalog contains one link, `#ac-2_smt.a.5`, that does not resolve to any
identifier in the document. `mizan validate` reports it as a single warning
(`oscal-resource-link-unresolved`). It is left as published.

## License

From `LICENSE.md` in usnistgov/oscal-content at the commit above:

> As a work of the United States government, this project is in the public
> domain within the United States.
>
> Additionally, we waive copyright and related rights in the work worldwide
> through the CC0 1.0 Universal public domain dedication.
