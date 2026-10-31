| target | method | id | cases | input (keys) | expected | actual | verdict | evidence |
|---|---|---|---|---|---|---|---|---|
| site | - | SITE-01 | MAC-LNG-21,MAC-LNG-28 | `all 44 relative href/src/poster and #anchors resolve from /, /en/ and 404` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-02 | MAC-LNG-20 | `<html lang> = vi / en and the VI\|EN header switch links to the equivalent page, aria-current set` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-03 | MAC-LNG-22 | `per-language title/description/canonical/hreflang(vi,en,x-default)/og:locale/theme-color; noindex only on 404` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-04 | MAC-LNG-23,MAC-DOC-01 | `vi and en pages carry the same sections, steps, FAQ, tables, screenshots, numeric facts and external links` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-05 | MAC-DOC-02,MAC-LNG-25 | `both Windows and macOS download links are present and visible without JS; script reads .exe and .dmg` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-06 | MAC-DOC-03,MAC-LNG-30 | `static fallbacks carry no stale version (product 0.2.7)` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-07 | MAC-DOC-01,MAC-DOC-04,MAC-DOC-05,MAC-DOC-12,MAC-LNG-24 | `macOS install sections (vi + en): no Windows-only claims, and dmg / Applications / Gatekeeper / Accessibility are explained` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-08/01 | MAC-DOC-01 | `stale-claim grep: 'no macOS build' claim` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-08/04 | MAC-DOC-04 | `stale-claim grep: left click toggles V/E stated without the Windows qualifier` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-08/06 | MAC-DOC-06 | `stale-claim grep: 'nine actions' (UI has 4 rows); F-keys settable in the Shortcuts window (they are config.toml only)` | `no findings` | `[F-keys settable in the Shortcuts window (they are config.toml only)] site/index.html: ... để trống. Muốn dùng vẫn đặt được trong Phím tắt…   ...; [F-keys settable in the Shortcuts window (they are config.toml only)] site/en/index.html: ...ey start unassigned. You can still set them under Shortcuts…   ...` | FAIL | scripts/mac-e2e/site-check.py |
| site | - | SITE-08/07 | MAC-DOC-07 | `stale-claim grep: raw-zone marking control (no such control)` | `no findings` | `[raw-zone marking control (no such control)] site/index.html: ... Vùng gõ thô Đánh dấu URL hoặc đoạn mã để VietKB không chuyển đ...; [raw-zone marking control (no such control)] site/en/index.html: ...3>Raw text zones Mark URLs or code so VietKB leaves them ex...` | FAIL | scripts/mac-e2e/site-check.py |
| site | - | SITE-08/08 | MAC-DOC-08 | `stale-claim grep: NFC/NFD choice without a Windows qualifier` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-08/09 | MAC-DOC-09 | `stale-claim grep: theme choice claimed without a platform qualifier` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-08/10 | MAC-DOC-10 | `stale-claim grep: 'three auxiliary windows' (there are four incl. About); 480x460 window size (Mac ~480x453)` | `no findings` | `['three auxiliary windows' (there are four incl. About)] site/index.html: ...d">Một cửa sổ cài đặt và ba cửa sổ phụ: Phím tắt, Bảng gõ tắt, ...; ['three auxiliary windows' (there are four incl. About)] site/en/index.html: ...>One settings window and three more: Shortcuts, Abbreviations...` | FAIL | scripts/mac-e2e/site-check.py |
| site | - | SITE-09 | MAC-DOC-01,MAC-DOC-05,MAC-LNG-24 | `Mac content present in both languages and in the 404 (title/meta, Mac screenshots, default shortcuts, autostart wording)` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-10 | MAC-LNG-29,MAC-LNG-06 | `English page has no untranslated Vietnamese (text, alt, aria-label, title); Vietnamese page has no English paragraphs` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-11 | MAC-LNG-21,MAC-DOC-22 | `Pages workflow uploads site/ (which contains en/), runs on site/** and manually` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| site | - | SITE-12 | MAC-DOC-01,MAC-DOC-05,MAC-DOC-23 | `README: macOS install (dmg, Gatekeeper, Accessibility), Mac autostart wording, vkb-macos row, Mac build` | `no findings` | `README structure lacks crates/vkb-macos` | FAIL | scripts/mac-e2e/site-check.py |
| site | - | SITE-13 | MAC-DOC-13,MAC-DOC-06,MAC-DOC-15 | `Mac release notes: Dock icon wording (M3), F-key claim, Input Monitoring mentioned` | `no findings` | `release notes claim F-keys are settable in Phím tắt…; release notes never mention Input Monitoring (MAC-DOC-15)` | FAIL | scripts/mac-e2e/site-check.py |
| site | - | SITE-14 | MAC-LNG-28,MAC-DOC-21 | `404.html is bilingual and every link/asset is root-absolute, so it renders under /en/ too` | `no findings` | `ok` | PASS | scripts/mac-e2e/site-check.py |
| strings | - | STR-01 | MAC-LNG-06 | `vi/en locales have identical keys (345 vi, 345 en), none empty, placeholders match` | `no findings` | `ok` | PASS | scripts/mac-e2e/strings-audit.py |
| strings | - | STR-02 | MAC-LNG-06 | `every locale key is used in ui/src (400 literal refs, 1 plural bases, prefixes [''])` | `no findings` | `1 unused: common.add` | FAIL | scripts/mac-e2e/strings-audit.py |
| strings | - | STR-03 | MAC-LNG-06 | `every literal t()/…Key/plural() key in ui/src exists in both locales` | `no findings` | `ok` | PASS | scripts/mac-e2e/strings-audit.py |
| strings | - | STR-04 | MAC-LNG-02,MAC-LNG-06 | `English locale values contain no Vietnamese diacritics (except the language endonym)` | `no findings` | `0: ` | PASS | scripts/mac-e2e/strings-audit.py |
| strings | - | STR-05 | MAC-LNG-02,MAC-LNG-06 | `no user-visible literal text/aria-label/title/placeholder/alt in ui/src markup (heuristic)` | `no findings` | `0: ` | PASS | scripts/mac-e2e/strings-audit.py |
| strings | - | STR-06 | MAC-LNG-02,MAC-LNG-06 | `no Vietnamese text in ui/src outside locales and comments (examples of typing output show up here for review)` | `no findings` | `1: ui/src/components/TitleBar.svelte:151 'present only while a newer release is known; opens Giới thiệu. -->'` | FAIL | scripts/mac-e2e/strings-audit.py |
| strings | - | STR-07 | MAC-I18-05,MAC-LNG-07 | `Rust i18n table (74 keys): complete pairs, English clean, keys used, tray menu equals doc 27` | `no findings` | `ok` | PASS | scripts/mac-e2e/strings-audit.py |
| strings | - | STR-08 | MAC-LNG-06 | `ui/scripts/i18n-check.mjs (project i18n test)` | `no findings` | `i18n ok: 345 keys, vi and en match` | PASS | scripts/mac-e2e/strings-audit.py |

| target | PASS | FAIL | other |
|---|---|---|---|
| site | 15 | 5 | 0 |
| strings | 6 | 2 | 0 |
