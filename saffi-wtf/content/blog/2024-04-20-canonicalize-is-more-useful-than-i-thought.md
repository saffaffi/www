---
tags = ["rust", "development", "meta"]
---

# `canonicalize()` Is More Useful Than I Thought

I only just realised today, as I was working on this site, that
[`Path::canonicalize()`][canonicalize] is more useful than I thought it was.

Maybe I was missing the point, I'm not sure. But it wasn't until now that I
found out it doesn't just resolve symlinks and remove redundant `..` and `.`
components in the middle of paths; it will make a path _absolute_.

I ran into this because I was working on a bit of this site's code that will
watch for file changes, so I can just save a content file and it will hot-reload
it instead of me having to rerun `cargo run` (this is by no means an original
idea, but in this case it was inspired by [fasterthanlime's writeup where he
talks about using `notify` to reload files][lime-site]).

The event handler checks to see whether the path is relative to the content
directory path:

```rust
match event.path.strip_prefix(&content_path) {
    Ok(relative) => {
        info!(path = ?relative, "received event for relative path");
    }
    Err(error) => {
        error!(
            path = ?event.path,
            content_path = ?content_path,
            %error,
            "event contains path not relative to content path"
        );
    }
}
```

But it was hitting the error case, which was weird because the watcher is only
watching for changes _inside the content path_:

```rust
debounced_watcher
    .watcher()
    .watch(self.content_path.as_std_path(), RecursiveMode::Recursive)
    .map_err(WatchPath)?;
```

And once I added the `content_path` field to that log event that you can see
above, I discovered the problem:

```text
2.860870208s ERROR www_saffi_wtf::state: event contains path not relative to content path path="/Users/saffi/src/github.com/saffaffi/www/saffi-wtf/content/blog/2024-04-20-canonicalize-is-more-useful-than-i-thought.md" content_path="./content" error=prefix not found
```

When I run the site in development, I run it with `$CONTENT_PATH` set to
`./content`, and of _course_ that big absolute path to the content file isn't
going to think it's relative to the little relative path `./content`. It turns
out the solution is really simple: call [`canonicalize()`][canonicalize] on the
content path at startup. That expands out the `.` at the beginning into its full
absolute path glory, because the current implementation of `canonicalize()` just
invokes [`realpath`][realpath] on Unixy platforms.

```text
3.423157667s  INFO www_saffi_wtf::state: received event for relative path path="blog/2024-04-20-canonicalize-is-more-useful-than-i-thought.md"
```

But my explanation isn't entirely accurate, because it _even works if you don't
provide the `.`_!

```text
[saffi-wtf/src/state.rs:74:32] content_path = "content"
[saffi-wtf/src/state.rs:74:27] dbg!(content_path).canonicalize_utf8() = Ok(
    "/Users/saffi/src/github.com/saffaffi/www/saffi-wtf/content",
)
```

That's pretty useful, and it's a nice change from the kinds of shenanigans you
have to get up to in (e.g.) Bash scripts sometimes in order to make paths
absolute.

[canonicalize]: https://doc.rust-lang.org/std/path/struct.Path.html#method.canonicalize
[lime-site]: https://fasterthanli.me/articles/a-new-website-for-2020
[realpath]: https://www.man7.org/linux/man-pages/man3/realpath.3.html

---
date = "2024-04-20"
---

This is just an additional test bit to try making this into a thread.
