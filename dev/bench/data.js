window.BENCHMARK_DATA = {
  "lastUpdate": 1770667668361,
  "repoUrl": "https://github.com/arcbox-labs/buildkit-client",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "github@sku.moe",
            "name": "AprilNEA",
            "username": "AprilNEA"
          },
          "committer": {
            "email": "github@sku.moe",
            "name": "AprilNEA",
            "username": "AprilNEA"
          },
          "distinct": true,
          "id": "33cbad21c445a0856be03f68e96413153260ca5e",
          "message": "ci: make benchmark output parseable",
          "timestamp": "2026-02-09T18:22:59Z",
          "tree_id": "c492416dc73154ccd3be0db72bdc49bbcc7084ac",
          "url": "https://github.com/arcbox-labs/buildkit-client/commit/33cbad21c445a0856be03f68e96413153260ca5e"
        },
        "date": 1770661672549,
        "tool": "cargo",
        "benches": [
          {
            "name": "platform_parse_simple",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_parse_with_variant",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_to_string",
            "value": 56,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_local",
            "value": 124,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_github",
            "value": 128,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_full_chain",
            "value": 542,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "session_creation",
            "value": 1094,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "session_metadata_generation",
            "value": 556,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "dockerfile_source_match_local",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "dockerfile_source_match_github",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_args_insertion",
            "value": 184,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "github@sku.moe",
            "name": "AprilNEA",
            "username": "AprilNEA"
          },
          "committer": {
            "email": "github@sku.moe",
            "name": "AprilNEA",
            "username": "AprilNEA"
          },
          "distinct": true,
          "id": "dcd99ee054901920bc382fff91bdbf69b9834439",
          "message": "chore: address RustSec audit findings",
          "timestamp": "2026-02-09T18:58:31Z",
          "tree_id": "1134119994d085b90507a586f2e93588f50e0f54",
          "url": "https://github.com/arcbox-labs/buildkit-client/commit/dcd99ee054901920bc382fff91bdbf69b9834439"
        },
        "date": 1770665024415,
        "tool": "cargo",
        "benches": [
          {
            "name": "platform_parse_simple",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_parse_with_variant",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "platform_to_string",
            "value": 59,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_local",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_github",
            "value": 124,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_full_chain",
            "value": 544,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "session_creation",
            "value": 1071,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "session_metadata_generation",
            "value": 560,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "dockerfile_source_match_local",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "dockerfile_source_match_github",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_args_insertion",
            "value": 183,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "github@sku.moe",
            "name": "AprilNEA",
            "username": "AprilNEA"
          },
          "committer": {
            "email": "github@sku.moe",
            "name": "AprilNEA",
            "username": "AprilNEA"
          },
          "distinct": true,
          "id": "9d0001b6640fcce44b944a5c93360c61c9d47926",
          "message": "build: vendor protos for offline builds",
          "timestamp": "2026-02-09T19:52:11Z",
          "tree_id": "7f65c6c44f2e20d8a2de14694b6b9ca8bf00b9e1",
          "url": "https://github.com/arcbox-labs/buildkit-client/commit/9d0001b6640fcce44b944a5c93360c61c9d47926"
        },
        "date": 1770667667689,
        "tool": "cargo",
        "benches": [
          {
            "name": "platform_parse_simple",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_parse_with_variant",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_to_string",
            "value": 53,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_local",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_github",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_full_chain",
            "value": 532,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "session_creation",
            "value": 1080,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "session_metadata_generation",
            "value": 564,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "dockerfile_source_match_local",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "dockerfile_source_match_github",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_args_insertion",
            "value": 182,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}