{
  # Keep NixCI focused on distributable Linux outputs.
  systems = [ "x86_64-linux" ];

  onlyBuild = [
    "packages.x86_64-linux.runtime"
    "packages.x86_64-linux.yazelix"
  ];
}
