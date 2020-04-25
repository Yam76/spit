# spit

`spit` allows you to abbreviate commonly typed phrases. This makes it easier to type such phrases for piping and prevents inconsistencies that arise from having to remember the exact phrase.

`spit` is a work-in-progress.

Sample usage:

```
spit --init
spit -a "[BUG]" b bug
spit -a "[FEATURE]" feat
spit -a "[FIX]" fix
spit b
spit bug
spit fix feat
```

See `spit --help` for more information.