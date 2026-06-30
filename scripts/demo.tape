# Demo GIF for README
# Requires: VHS (github.com/charmbracelet/vhs)
# 
# On macOS: brew install vhs
# On Linux: go install github.com/charmbracelet/vhs@latest
#
# Alternative: use scripts/record-demo.sh (uses asciinema + agg)

Output assets/demo.gif

Set FontSize 16
Set Width 800
Set Height 400
Set TypingSpeed 12ms

Type "cargo run --example basic" Enter
Sleep 5s
