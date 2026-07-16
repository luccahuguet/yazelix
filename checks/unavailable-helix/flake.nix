{
  inputs.nixpkgs.follows = "nixpkgs";
  outputs = _: throw "yazelix-no-helix evaluated the managed Helix input";
}
