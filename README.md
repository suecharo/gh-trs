# gh-trs: your own API fully based on GitHub to serve workflow metadata

[![Apache License](https://img.shields.io/badge/license-Apache%202.0-orange.svg?style=flat&color=important)](http://www.apache.org/licenses/LICENSE-2.0)

Global Alliance for Genomics and Health (GA4GH) Cloud Workstream has created Workflow Execution Service (WES) and Tool Registry Service (TRS) standard definitions for executing and sharing workflows. Based on our experience in developing WES, the current TRS definition lacks information such as workflow attachments (e.g., configuration files and database files, etc.) and workflow parameter templates (e.g., required inputs and their type information). Therefore, there is a problem that workflows cannot be executed even if the TRS URL is specified. Also, there are existing TRSs (e.g., Dockstore, BioContainers, etc.) that use GitHub as the registry entity. Here, we propose a TRS publication protocol by combining GitHub (file hosting, user authentication, and version management), GitHub Actions (continuous testing, workflow analysis), and GitHub Pages (REST API hosting). This allows users to retrieve information for workflow execution from the GitHub repository hosting the workflow documents via TRS definitions.

## Installation

A binary for linux is available.

## Usage

```
/gh-trs --help
gh-trs 0.1.0

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
        --repo-url <repo-url>          GitHub repository URL (default: URL of the git repository you are in) [default: ]
    -t, --tag <tag>                    Add tag to commit [default: ]
        --user-email <user-email>      User email used for git commit (defaults to the git config) [default: ]
        --user-name <user-name>        User name used for git commit (defaults to the git config) [default: ]

ARGS:
    <config-file>    Path or URL to the gh-trs config file [default: gh-trs.yml]
```

## Outline drawing

[gh-trs Outline Drawing](https://i.imgur.com/aP5hnQS.png)

## Acknowledgement

The gh-trs is partially supported by JSPS KAKENHI Grant Numbers 20J22439.

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE](https://github.com/suecharo/gh-trs/blob/master/LICENSE).
