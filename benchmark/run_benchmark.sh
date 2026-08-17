#!/bin/bash

echo "Running benchmarks for Jolt vs Maven vs Gradle..."
echo "This test measures the time to resolve the following dependencies:"
echo "- com.google.code.gson:gson:2.10.1"
echo "- org.slf4j:slf4j-api:2.0.7"
echo "- com.fasterxml.jackson.core:jackson-databind:2.15.2"
echo ""

# We use hyperfine to run 2 warmup iterations and then 5 measured runs.
# Results will be exported to results.md
hyperfine --warmup 2 --runs 5 \
  "cd jolt-test && jolt install" \
  "cd maven-test && mvn dependency:resolve" \
  "cd gradle-test && gradle dependencies" \
  --export-markdown results.md

echo "Benchmark complete! Results saved in benchmark/results.md"
