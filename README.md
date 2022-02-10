# gh-trs: CLI tool to publish and test your own GA4GH TRS API using GitHub

[![Apache License](https://img.shields.io/badge/license-Apache%202.0-orange.svg?style=flat&color=important)](http://www.apache.org/licenses/LICENSE-2.0)
[![test](https://github.com/suecharo/gh-trs/actions/workflows/tarpaulin.yml/badge.svg?branch=main)](https://github.com/suecharo/gh-trs/actions/workflows/tarpaulin.yml)

CLI tool for publishing workflows as [GA4GH TRS API](https://www.ga4gh.org/news/tool-registry-service-api-enabling-an-interoperable-library-of-genomics-analysis-tools/) and testing workflows using GitHub.

As feature:

- Generating templates for publishing from workflow document's URL
- Testing workflows based on the registration file
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

First, generate a template of the registration file from the GitHub location of the workflow document as:

```bash
$ gh-trs make-template https://github.com/suecharo/gh-trs/blob/main/tests/CWL/wf/trimming_and_qc.cwl
```

[`test_config_CWL_template.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_CWL.yml) is an example of what will be generated.

Next, edit the generated `./gh-trs-config.yml` as [`test_config_CWL.yml`](https://github.com/suecharo/gh-trs/blob/main/tests/test_config_CWL.yml).

The main part to edit is below:

- `workflow.files`: the list of files to be included in the workflow
- `workflow.testing`: the list of tests to be run

Then, generate the GA4GH TRS API based on the registration file and deploy it on GitHub Pages as:

```bash
$ gh-trs publish --repo <repo_owner>/<repo_name> ./gh-trs-config.yml
```

Deployed workflows can be retrieved in the [GA4GH TRS API specs](https://editor.swagger.io/?url=https://raw.githubusercontent.com/ga4gh/tool-registry-schemas/develop/openapi/openapi.yaml) as:

```bash
$ curl -L https://<repo_owner>.github.io/<repo_name>/tools
```

## Acknowledgement

The gh-trs is partially supported by JSPS KAKENHI Grant Numbers 20J22439.

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0). See the [LICENSE](https://github.com/suecharo/gh-trs/blob/main/LICENSE).
