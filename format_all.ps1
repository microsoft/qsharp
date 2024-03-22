Get-ChildItem -Recurse .\samples -Filter *.qs |
ForEach-Object {
  #cargo run --bin qsc_formatter $_.FullName
  .\target\release\qsc_formatter.exe $_.FullName
}

Get-ChildItem -Recurse .\library -Filter *.qs |
ForEach-Object {
  #cargo run --bin qsc_formatter $_.FullName
  .\target\release\qsc_formatter.exe $_.FullName
}

