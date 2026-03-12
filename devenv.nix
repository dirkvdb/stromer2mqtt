{
  pkgs,
  inputs,
  ...
}:
{
  overlays = [
    (import inputs.rust-overlay)
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
  };
  languages.cplusplus.enable = true;

  packages = with pkgs; [
    just
    lld
    cargo-nextest
  ];
}
