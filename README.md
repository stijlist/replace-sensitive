# replace-sensitive

Replace Stringaling/stringaling/StringALing/string_a_ling/string-a-ling with Zingaling/zingaling/ZingALing/zing_a_ling/zing-a-ling.

Given a regex, turn it into a set of regexes that match the patterns above.

s/hello/hi/g -> {s/hello/hi/g, s/Hello/Hi/g}

s/hello_world/hi_world/g
  ->
{hello_world => hi_world, Hello_World => Hi_World HELLO_WORLD => HI_WORLD, HelloWorld => HiWorld, helloWorld, hiWorld, hello-world => hi-world}
