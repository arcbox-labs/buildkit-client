window.BENCHMARK_DATA = {
  "lastUpdate": 1770669586894,
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
          "id": "b39cfbb96a088695b4ca05912c4a2002b4eddd81",
          "message": "ci: add release-plz",
          "timestamp": "2026-02-09T20:10:40Z",
          "tree_id": "feb3c5586741be12904f1772516ed33223330e25",
          "url": "https://github.com/arcbox-labs/buildkit-client/commit/b39cfbb96a088695b4ca05912c4a2002b4eddd81"
        },
        "date": 1770668138357,
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
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_to_string",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_local",
            "value": 125,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_github",
            "value": 127,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_full_chain",
            "value": 540,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "session_creation",
            "value": 1071,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "session_metadata_generation",
            "value": 546,
            "range": "± 2",
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
      },
      {
        "commit": {
          "author": {
            "email": "41898282+github-actions[bot]@users.noreply.github.com",
            "name": "github-actions[bot]",
            "username": "github-actions[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "659a10d7acc75e2d341c955d64eb209c160fb741",
          "message": "chore: release (#2)\n\nCo-authored-by: github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>",
          "timestamp": "2026-02-10T04:14:18+08:00",
          "tree_id": "76b229f38893374e440ff26062b2430367f1f44a",
          "url": "https://github.com/arcbox-labs/buildkit-client/commit/659a10d7acc75e2d341c955d64eb209c160fb741"
        },
        "date": 1770668320131,
        "tool": "cargo",
        "benches": [
          {
            "name": "platform_parse_simple",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_parse_with_variant",
            "value": 90,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "platform_to_string",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_local",
            "value": 124,
            "range": "± 2",
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
            "value": 542,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "session_creation",
            "value": 1070,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "session_metadata_generation",
            "value": 551,
            "range": "± 2",
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
          "id": "3624b0410aa718c7f8cf86fd38ef2cc59e2f7798",
          "message": "ci: install protoc for release-plz",
          "timestamp": "2026-02-09T20:26:26Z",
          "tree_id": "c9b8f5f2104fb30ca70abc7607ddb7968b0c5254",
          "url": "https://github.com/arcbox-labs/buildkit-client/commit/3624b0410aa718c7f8cf86fd38ef2cc59e2f7798"
        },
        "date": 1770669586584,
        "tool": "cargo",
        "benches": [
          {
            "name": "platform_parse_simple",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_parse_with_variant",
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "platform_to_string",
            "value": 55,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "build_config_local",
            "value": 125,
            "range": "± 3",
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
            "value": 529,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "session_creation",
            "value": 1072,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "session_metadata_generation",
            "value": 566,
            "range": "± 2",
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
      }
    ]
  }
}