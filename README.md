# gh-trs: CLI tool to publish and test your own GA4GH TRS API using GitHub

[![Apache License](https://img.shields.io/badge/license-Apache%202.0-orange.svg?style=flat&color=important)](http://www.apache.org/licenses/LICENSE-2.0)
[![test](https://github.com/suecharo/gh-trs/actions/workflows/tarpaulin.yml/badge.svg?branch=main)](https://github.com/suecharo/gh-trs/actions/workflows/tarpaulin.yml)

CLI tool for publishing workflows as the [GA4GH Tool Registry Service (TRS) API](https://www.ga4gh.org/news/tool-registry-service-api-enabling-an-interoperable-library-of-genomics-analysis-tools/) and testing workflows using GitHub.

As feature:

- Generating templates for publishing from workflow document's URL (called `config_file`)
- Testing workflows based on the gh-trs configuration file
- Publishing workflows to GitHub as GA4GH TRS API

## Installation

Use a single binary that is built without any dependencies:

```bash
$ curl -fsSL -O https://github.com/suecharo/gh-trs/releases/latest/download/gh-trs
$ chmod +x ./gh-trs
$ ./gh-trs --help
```

Or, use `Docker` environment (also `docker-compose`):

```bash
$ docker-compose up -d --build
$ docker-compose exec app gh-trs --help
```

## Getting started

First, the `gh-trs` needs the `GitHub Personal Access Token` for various operations through GitHub REST API.
Please refer to [GitHub Docs - Creating a personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token) for how to generate the `GitHub Personal Access Token`.

The required scopes are as follows (also see ScreenShot):

- `repo - public_repo`
- `user - read:user`

<img src="https://user-images.githubusercontent.com/26019402/149902689-bfd4707d-9792-41fd-b22f-8a1631489399.png" alt="gh-trs-img-1" width="600">

Once you have generated the `GitHub Personal Access Token`, you need to pass the `gh-trs` it in one of the following ways:

- env file: write the token to `.env` file like `GITHUB_TOKEN=<paste_your_token>`
- environment variable: set the `GITHUB_TOKEN` environment variable
- command-line option: use `--gh-token <paste_your_token>` option

---

Use the workflow [`trimming_and_qc.cwl`](https://github.com/suecharo/gh-trs/blob/main/tests/CWL/wf/trimming_and_qc.cwl) as an example.

First, generate a template of the gh-trs configuration file from the GitHub location of the workflow document as:

```bash
$ gh-trs make-template https://github.com/suecharo/gh-trs/blob/main/tests/CWL/wf/trimming_and_qc.cwl
```

[`test_config_CWL_template.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_CWL.yml) is an example of what will be generated.

Next, edit the generated `./gh-trs-config.yml` as [`test_config_CWL.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_CWL.yml).

The main part to edit is below:

- `workflow.files`: the list of files to be included in the workflow
- `workflow.testing`: the list of tests to be run

See [readme - validate](https://github.com/suecharo/gh-trs#validate) for more details.

Then, generate the GA4GH TRS API based on the gh-trs configuration file and deploy it on GitHub Pages as:

```bash
$ gh-trs publish --repo <repo_owner>/<repo_name> ./gh-trs-config.yml
```

Deployed workflows can be retrieved in the [GA4GH TRS API specs](https://editor.swagger.io/?url=https://raw.githubusercontent.com/ga4gh/tool-registry-schemas/develop/openapi/openapi.yaml) as:

```bash
$ curl -L https://<repo_owner>.github.io/<repo_name>/tools
```

## Usage

This section describes some of the subcommands of the `gh-trs`.

```bash
$ gh-trs --help
gh-trs 0.1.1
CLI tool to publish and test your own GA4GH TRS API using GitHub

USAGE:
    gh-trs <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help             Prints this message or the help of the given subcommand(s)
    make-template    Make a template for the gh-trs configuration file
    publish          Publish the TRS response to GitHub
    test             Test the workflow based on the gh-trs configuration file
    validate         Validate the gh-trs configuration file
```

### make-template

Generate a template of the gh-trs configuration file from the GitHub location of the primary workflow file.

```bash
$ gh-trs make-template --help
gh-trs-make-template 0.1.1
Make a template for the gh-trs configuration file

USAGE:
    gh-trs make-template [FLAGS] [OPTIONS] <workflow-location>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode

OPTIONS:
        --gh-token <github-token>    GitHub Personal Access Token
    -o, --output <output>            Path to the output file [default: gh-trs-config.yml]

ARGS:
    <workflow-location>    Location of the primary workflow document
```

Only URLs hosted on GitHub are accepted for the `workflow-location`.
This URL is a URL like `https://github.com/suecharo/gh-trs/blob/main/tests/CWL/wf/trimming_and_qc.cwl`, and it will be converted to a raw URL like `https://raw.githubusercontent.com/suecharo/gh-trs/645a193826bdb3f0731421d4ff1468d0736b4a06/tests/CWL/wf/trimming_and_qc.cwl` later.

The `gh-trs` collects various information and generates a template for the gh-trs configuration file.
In particular, `workflow.files` will be generated a file list from the primary workflow location recursively.

### validate

Validate the schema and contents of the gh-trs configuration file.

```bash
$ gh-trs validate --help
gh-trs-validate 0.1.1
Validate the gh-trs configuration file

USAGE:
    gh-trs validate [FLAGS] [OPTIONS] [config-locations]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode

OPTIONS:
        --gh-token <github-token>    GitHub Personal Access Token

ARGS:
    <config-locations>...    Location of the gh-trs configuration files (local file path or remote URL) [default:
                             gh-trs-config.yml]
```

An explanation of the validation rules for some fields in the gh-trs configuration file is following:

- `id`: ID of the workflow. The `make-template` command generates it. If you want to update an existing workflow, fill in the ID of the existing workflow.
- `version`: Version in the form `x.y.z`.
- `authors`: List of authors.
- `workflow.name`: Please fill freely. It must be alphanumeric or contain \_, -, or space.
- `workflow.readme`: It is used to `describe` field of the workflow. Use any URL you like.
- `workflow.language`: `CWL`, `WDL`, `NFL`, and `SMK` are supported.
- `workflow.files`: The list of files. Files specified as `type: secondary` will be placed in the execution directory with `target` as the path at workflow execution time.
- `workflow.testing`: The list of tests. Please refer to `test` for how to write tests.

Several example are prepared. Please check:

- [`test_config_CWL.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_CWL.yml)
- [`test_config_WDL.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_WDL.yml)
- [`test_config_NFL.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_NFL.yml)
- [`test_config_SMK.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_SMK.yml)

### test

Test the workflow based on the configuration file.

```bash
$ gh-trs test --help
gh-trs-test 0.1.1
Test the workflow based on the gh-trs configuration file

USAGE:
    gh-trs test [FLAGS] [OPTIONS] [config-locations]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode

OPTIONS:
    -d, --docker-host <docker-host>      Location of the docker host [default: unix:///var/run/docker.sock]
        --gh-token <github-token>        GitHub Personal Access Token
    -w, --wes-location <wes-location>    Location of WES in which to run the test. If not specified, `sapporo-service`
                                         will be started

ARGS:
    <config-locations>...    Location of the gh-trs configuration files (local file path or remote URL) [default:
                             gh-trs-config.yml]
```

The test is run using the GA4GH Workflow Execution Service (WES; [GA4GH - WES API](https://www.ga4gh.org/news/ga4gh-wes-api-enables-portable-genomic-analysis/).
In particular, the `gh-trs` use the [`sapporo-service`](https://github.com/sapporo-wes/sapporo-service) as a WES.
If the option `--wes-location` is not specified, `sapporo-service` will be stated using the default `DOCKER_HOST`.

An example of the `workflow.testing` field in the config file is shown below:

```yaml
testing:
  - id: test_1
    files:
      - url: "https://example.com/path/to/wf_params.json"
        target: wf_params.json
        type: wf_params
      - url: "https://example.com/path/to/wf_engine_params.json"
        target: wf_engine_params.json
        type: wf_engine_params
      - url: "https://example.com/path/to/data.fq"
        target: data.fq
        type: other
```

There are three types of file types:

- `wf_params`: The parameters for the workflow.
- `wf_engine_params`: The execution parameters for the workflow engine.
- `other`: Other files (e.g., data files).

The files specified as `wf_params` and `wf_engine_params` will be placed as WES execution parameters at the WES runtime.
Also, `other` files will be placed in the execution directory with `target` as the path at workflow execution time.

You can freely specify the `id` field.

For more information on how to run WES, please refer to the [WES API document](https://editor.swagger.io/?url=https://ga4gh.github.io/workflow-execution-service-schemas/openapi.yaml) and the [sapporo document](https://github.com/sapporo-wes/sapporo-service).

### publish

Publish workflows to GitHub as GA4GH TRS API.

```bash
$ gh-trs publish --help
gh-trs-publish 0.1.1
Publish the TRS response to GitHub

USAGE:
    gh-trs publish [FLAGS] [OPTIONS] --repo <repo> [config-locations]...

FLAGS:
        --from-trs     Recursively get the gh-trs configuration files from the TRS endpoint and publish them. It is
                       mainly intended to be tested and published all at once in a CI environment. If you use this option,
                       specify the TRS endpoint for `config_location`
    -h, --help         Prints help information
    -V, --version      Prints version information
    -v, --verbose      Verbose mode
        --with-test    Test before publishing

OPTIONS:
    -b, --branch <branch>                GitHub branch to publish to [default: gh-pages]
    -d, --docker-host <docker-host>      Location of the docker host [default: unix:///var/run/docker.sock]
        --gh-token <github-token>        GitHub Personal Access Token
    -r, --repo <repo>                    GitHub Repository to publish to. (e.g. owner/name)
    -w, --wes-location <wes-location>    Location of WES in which to run the test. If not specified, `sapporo-service`
                                         will be started

ARGS:
    <config-locations>...    Location of the gh-trs configuration files (local file path or remote URL) [default:
                             gh-trs-config.yml]
```

GA4GH TRS responses will be generated based on the gh-trs configuration file and published to GitHub Pages.
Also, with the `--repo <repo>` and `--branch <branch>` options, the `gh-trs` can specify the GitHub repository or branch to publish to.

The `gh-trs` can run tests before publishing using the `--with-test` option.
The tested workflows will have the `verified` field set to `true` in the TRS response.

The `gh-trs` can get the gh-trs configuration files from the TRS endpoint and publish them using the `--from-trs` option.
Therefore, if you want to test and publish all the workflows of an already published TRS, run a command like:

```bash
$ gh-trs publish --repo <owner/name> --branch gh-pages --with-test --from-trs https://example.com/path/to/trs
```

## Continuous testing (CI/CD)

The GitHub Action ([`actions/gh-trs-action`](https://github.com/marketplace/actions/gh-trs-action?version=v1)) for continuous testing are published.

The inputs of this action are the following:

- `gh-token`: GitHub Personal Access Token
- `repo`: **Optional** GitHub repository to publish to. (e.g., `owner/name`, default: your repository)
- `branch`: **Optional** GitHub branch to publish to. (default: `gh-pages`)
- `trs-endpoint`: **Optional** TRS endpoint to get the gh-trs configuration files. (default: your default trs endpoint)

If you want to specify these inputs, use the `with` context (docs., https://docs.github.com/ja/actions/using-workflows/workflow-syntax-for-github-actions#) like:

```yaml
jobs:
  test-and-publish:
    steps:
      - name: gh-trs-action
        uses: suecharo/gh-trs-action@v1
        with:
          gh-token: ${{ secrets.GITHUB_TOKEN }}
          repo: suecharo/gh-trs
          branch: gh-pages
          trs-endpoint: https://suecharo.github.io/gh-trs/
```

These inputs are **Optional**, and if not specified, default values based on your repository will be used.

In this action, the following commands will be executed:

```bash
$ gh-trs publish --verbose --with-test --repo ${{ inputs.repo }} --branch ${{ inputs.branch }} --from-trs ${{ inputs.trs-endpoint }}
```

The test results will then be uploaded to GitHub Actions as an artifact named `gh-trs-test-logs`.
Also, if the tests are run is published as CI, the URL of the relevant run of GitHub Actions will be set in the `verified_source` field in the TRS response.

Below we provide the recipes for the two patterns of GitHub Actions.

### Page build trigger

This is a recipe for running CI in response to local execution.

```yaml
name: page-build-ci

on:
  page_build: {}

jobs:
  test-and-publish:
    runs-on: ubuntu-latest
    if: "! contains(github.event.head_commit.message, 'in CI')"
    steps:
      - name: gh-trs-action
        uses: suecharo/gh-trs-action@v1
        with:
          gh-token: ${{ secrets.GITHUB_TOKEN }}
```

With this action is placed, when a command like the one below is executed in the local environment, CI will be launched:

```bash
$ gh-trs publish --repo suecharo/gh-trs ./tests/test_config_CWL.yml
```

### Schedule trigger

```yaml
name: schedule-ci

on:
  schedule:
    - cron: "0 0 * * 0" // every Sunday at midnight

jobs:
  test-and-publish:
    runs-on: ubuntu-latest
    steps:
      - name: gh-trs-action
        uses: suecharo/gh-trs-action@v1
        with:
          gh-token: ${{ secrets.GITHUB_TOKEN }}
```

With this action is placed, the CI will be executed based on the schedule.

## Acknowledgement

The `gh-trs` is partially supported by JSPS KAKENHI Grant Numbers 20J22439.

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0).
See the [LICENSE](https://github.com/suecharo/gh-trs/blob/main/LICENSE).
