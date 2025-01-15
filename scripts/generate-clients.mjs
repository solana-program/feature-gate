#!/usr/bin/env zx
import "zx/globals";
import * as c from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor as renderJavaScriptVisitor } from "@codama/renderers-js";
import { renderVisitor as renderRustVisitor } from "@codama/renderers-rust";
import { getAllProgramIdls, getToolchainArgument } from "./utils.mjs";

// Instanciate codama.
const [idl, ...additionalIdls] = getAllProgramIdls().map(idl => rootNodeFromAnchor(require(idl)))
const codama = c.createFromRoot(idl, additionalIdls);

// Update programs.
codama.update(
  c.updateProgramsVisitor({
    "solanaFeatureGateProgram": { name: "solanaFeatureGate" },
  })
);

// Render JavaScript.
const jsClient = path.join(__dirname, "..", "clients", "js");
codama.accept(
  renderJavaScriptVisitor(path.join(jsClient, "src", "generated"), { 
    prettier: require(path.join(jsClient, ".prettierrc.json"))
  })
);

// Render Rust.
const rustClient = path.join(__dirname, "..", "clients", "rust");
codama.accept(
  renderRustVisitor(path.join(rustClient, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClient,
    toolchain: getToolchainArgument('format')
  })
);
