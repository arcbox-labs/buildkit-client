window.BENCHMARK_DATA = {
  "lastUpdate": 1770661672841,
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
      }
    ]
  }
}