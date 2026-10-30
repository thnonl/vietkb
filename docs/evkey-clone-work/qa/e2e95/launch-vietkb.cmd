@echo off
cd /d E:\vietkb
set RUST_LOG=vkb=debug
set VKB_DEBUG_KEYS=1
set RUST_BACKTRACE=1
echo ==== LAUNCH %DATE% %TIME% ==== >> "E:\vietkb\docs\evkey-clone-work\qa\e2e95\app.log"
"E:\vietkb\target\release\vietkb.exe" >> "E:\vietkb\docs\evkey-clone-work\qa\e2e95\app.log" 2>&1
echo ==== EXIT %ERRORLEVEL% %DATE% %TIME% ==== >> "E:\vietkb\docs\evkey-clone-work\qa\e2e95\app.log"
