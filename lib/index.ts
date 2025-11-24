import fs from "fs";
import path from "path";
import toml from "toml";

const content = path.join(__dirname, "config");

export type Alphabet = {};

const alphabet = toml.parse(
  fs.readFileSync(path.join(content, "alphabet.toml"), "utf-8")
);
