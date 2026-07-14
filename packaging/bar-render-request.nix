{
  coreutils,
  nushell,
  runtimeIdentity,
  zellijBar,
}: {
  widgetTray,
  shellLabel,
}: {
  zjstatus_plugin_url = "file:${zellijBar}/${zellijBar.wasmPath}";
  widget_tray = widgetTray;
  widget_frame = "none";
  widget_separator = "dot";
  editor_label = "hx";
  shell_label = shellLabel;
  terminal_label = "mars";
  custom_text = "";
  appearance_mode = "dark";
  tab_label_mode = "full";
  nu_bin = "${nushell}/bin/nu";
  yzx_control_bin = "${coreutils}/bin/false";
  yazelix_zellij_bar_widget_bin = "${zellijBar}/${zellijBar.widgetPath}";
  runtime_dir = "${runtimeIdentity}";
  claude_usage_display = "both";
  claude_usage_periods = ["5h" "week"];
  codex_usage_display = "quota";
  codex_usage_periods = ["5h" "week"];
  opencode_go_usage_display = "both";
  opencode_go_usage_periods = ["5h" "week" "month"];
}
