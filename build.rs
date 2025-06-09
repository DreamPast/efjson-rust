pub fn main() {
  cc::Build::new().file("c_src/build.c").compile("efjsonc");
}
