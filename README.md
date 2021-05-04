# gh-trs

[Test 用 repository](https://github.com/suecharo/gh-trs-test-repo)
[GitHub - tschaub/gh-pages](https://github.com/tschaub/gh-pages)

## usage

workflow とか `gh-trs.yml` がある git repository で `gh-trs` command で、`gh-pages` branch に REST API, github actions を deploy する

---

```
cwltool --validate
cwltool --make-template
cwltool --pack
cwltool --graph
```

実際の CI 処理

---

deploy した後、もう一度 github actions が回る。それまでは status pending にしておく。

## エラー処理

- `branch already exists`
  - `gh-pages` branch が既に存在している場合

## gh-pages

```
ubuntu@dh236 ~> gh-pages --help
Usage: gh-pages [options]

Options:
  -V, --version            output the version number
  -d, --dist <dist>        Base directory for all source files
  -s, --src <src>          Pattern used to select which files to publish (default: "**/*")
  -b, --branch <branch>    Name of the branch you are pushing to (default: "gh-pages")
  -e, --dest <dest>        Target directory within the destination branch (relative to the root) (default: ".")
  -a, --add                Only add, and never remove existing files
  -x, --silent             Do not output the repository url
  -m, --message <message>  commit message (default: "Updates")
  -g, --tag <tag>          add tag to commit
  --git <git>              Path to git executable (default: "git")
  -t, --dotfiles           Include dotfiles
  -r, --repo <repo>        URL of the repository you are pushing to
  -p, --depth <depth>      depth for clone (default: 1)
  -o, --remote <name>      The name of the remote (default: "origin")
  -u, --user <address>     The name and email of the user (defaults to the git config).  Format is "Your Name <email@example.com>".
  -v, --remove <pattern>   Remove files that match the given pattern (ignored if used together with --add). (default: ".")
  -n, --no-push            Commit only (with no push)
  -f, --no-history         Push force new commit without parent history
  --before-add <file>      Execute the function exported by <file> before "git add"
  -h, --help               output usage information
```

## GitHub pages rest api

- https://wiredcraft.com/blog/static-rest-apis-on-github-pages/
- https://towardsdatascience.com/using-github-pages-for-creating-global-api-76b296c4b3b5
  - json を置く

commit id を directory にする案
directory 構造で切り替える

`.nojekyll`

https://github.com/Kanahiro/gh-pages-rest-api

`index.html` として置く (中身は json)
