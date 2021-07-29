# gh-trs: your own API fully based on GitHub to serve workflow metadata

[![Apache License](https://img.shields.io/badge/license-Apache%202.0-orange.svg?style=flat&color=important)](http://www.apache.org/licenses/LICENSE-2.0)

![gh-trs Outline Drawing](https://i.imgur.com/aP5hnQS.png)

**_gh-trs is now pre-release version._**

## What is this?

A command line tool to expand the idea of [GA4GH TRS](https://github.com/ga4gh/tool-registry-service-schemas), and to deploy TRS on personal GitHub.

- Motivation
  - Want to provide workflow document analysis results (e.g., workflow parameters template and workflow grapg) via TRS API.
    - From our experience of creating a WES named [Sapporo](https://github.com/ddbj/sapporo)
  - Want to deploy TRS on personal GitHub.

## Installation

Prerequisites

- Linux
- Git `>=1.9`
- GitHub repository

Using `cargo`

```bash
$ git clone https://github.com/suecharo/gh-trs.git
$ cd gh-trs
$ cargo build --release
$ ./target/release/gh-trs -V
gh-trs 0.1.1
```

Using binary

```bash
$ curl -fsSL -O https://github.com/suecharo/gh-trs/releases/download/0.1.1/gh-trs
$ chmod +x gh-trs
$ ./gh-trs -V
gh-trs 0.1.1
```

## Usage

Specify a configuration file (e.g. [gh-trs.test.yml](./tests/gh-trs.test.yml)) with workflows/tools you want to deploy, and deploy TRS to the `gh-pages` branch of the GitHub repository.

```bash
$ gh-trs ./tests/gh-trs.test.yml
Cloning https://github.com/suecharo/gh-trs.git into "/tmp/tc40-0"
Checking out origin/gh-pages
Generating the TRS responses
Adding all
Committing as suecharo <suehiro619@gmail.com>
Pushing
Username for 'https://github.com': suecharo
Password for 'https://suecharo@github.com': <your-password>
Your TRS has been deployed to https://suecharo.github.io"/gh-trs"
Please check `curl -X GET https://suecharo.github.io"/gh-trs"/service-info/

$ curl -X GET https://suecharo.github.io"/gh-trs"/service-info/
{"id":"io.github.suecharo","name":"suecharo/gh-trs","type":{"group":"io.github.suecharo","artifact":"TRS","version":"gh-trs-1.0.0"},"description":"Generated by gh-trs.","organization":{"name":"suecharo","url":"https://github.com/suecharo"},"contact_url":"mailto:suehiro619@gmail.com","documentation_url":"https://suecharo.github.io/gh-trs","created_at":"2021-05-13T10:22:06Z","updated_at":"2021-05-13T10:22:06Z","environment":"prod","version":"20210513"}
```

---

The detailed options are as follows;

```bash
gh-trs 0.1.1
your own API fully based on GitHub to serve workflow metadata

USAGE:
    gh-trs [OPTIONS] [config-file]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --branch <branch>              Name of the branch you are pushing to [default: gh-pages]
        --dest <dest>                  Target directory within the destination branch (relative to the root) [default:
                                       .]
        --environment <environment>    Environment the service is running in. Suggested values are prod, test, dev,
                                       staging [default: prod]
        --git <git>                    Path to git executable [default: git]
    -m, --message <message>            Commit message [default: 'Updates by gh-trs.']
    -r, --remote <remote>              Name of the remote [default: origin]
        --repo-url <repo-url>          GitHub repository URL (default: URL of the git repository you are in)
    -s, --scheme <scheme>              Scheme of the repository URL to use in the directory to clone [default:
                                       Https]  [possible values: Https, Ssh]
    -t, --tag <tag>                    Add tag to commit
        --user-email <user-email>      User email used for git commit (defaults to the git config)
        --user-name <user-name>        User name used for git commit (defaults to the git config)

ARGS:
    <config-file>    Path or URL to the gh-trs config file [default: gh-trs.yml]
```

[Swagger viewer link](https://swagger-url.vercel.app/?url=https%3A%2F%2Fraw.githubusercontent.com%2Fsuecharo%2Fgh-trs%2Fdevelop%2Fgh-trs.openapi.yml)

## Acknowledgement

The gh-trs is partially supported by JSPS KAKENHI Grant Numbers 20J22439.

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE](https://github.com/suecharo/gh-trs/blob/master/LICENSE).
