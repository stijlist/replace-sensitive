# replace-sensitive

Replace an identifier and all "variants" of it (e.g. camelCase, PascalCase,
snake_case, kebab-case, TITLE_CASE, etc.) with a different identifier and all
"variants" of it.

    $ replace-sensitive string_a_ling zing_a_ling
    $ replace-sensitive stringALing zingALing

When an identifier only has one "token", it works just like sed, so

    $ replace-sensitive hello hi

is equivalent to

    $ sed -e "s/hello/hi/g"

and

    $ replace_sensitive hello_world hi_world

is like

    $ sed -e "s/hello_world/hi_world/g" | 
        sed -e "s/Hello_World/Hi_World/g" |
        sed -e "s/HELLO_WORLD/HI_WORLD/g" |
        sed -e "s/HelloWorld/HiWorld/g" |
        sed -e "s/helloWorld/hiWorld/g" |
        sed -e "s/hello-world/hi-world/g"

All string replacement is streaming so replace-sensitve is safe to use in the
unix filter style, just like sed.

