import { describe, expect, test } from "bun:test";
import hljs from "highlight.js/lib/core";
import zig from "./zig";

hljs.registerLanguage("zig", zig);

const render = (code: string): string =>
  hljs.highlight(code, { language: "zig" }).value;

describe("zig grammar", () => {
  test("numbers use the theme-styled hljs-number class", () => {
    const html = render("const x = 1024;");
    expect(html).toContain('<span class="hljs-number">1024</span>');
    expect(html).not.toContain("hljs-numbers");
  });

  test("every integer base is recognised", () => {
    expect(render("const a = 0xFF_FF;")).toContain(
      '<span class="hljs-number">0xFF_FF</span>',
    );
    expect(render("const a = 0o755;")).toContain(
      '<span class="hljs-number">0o755</span>',
    );
    expect(render("const a = 0b1010_1010;")).toContain(
      '<span class="hljs-number">0b1010_1010</span>',
    );
  });

  test("hexadecimal floats are matched whole", () => {
    expect(render("const y = 0x1.8p3;")).toContain(
      '<span class="hljs-number">0x1.8p3</span>',
    );
  });

  test("literals do not match inside identifiers", () => {
    const html = render("const n = nullable_thing + undefined_thing;");
    expect(html).not.toContain("hljs-literal");
    expect(render("const n = null;")).toContain(
      '<span class="hljs-literal">null</span>',
    );
  });

  test("array indexing is not mistaken for a C pointer", () => {
    expect(render("const v = arr[c];")).not.toContain("hljs-operator");
    expect(render("const p: [*c]u8 = ptr;")).toContain(
      '<span class="hljs-operator">[*c]</span>',
    );
  });

  test("keywords added after 0.8 are highlighted", () => {
    expect(render("const Foo = opaque {};")).toContain(
      '<span class="hljs-keyword">opaque</span>',
    );
    expect(render("const P = *addrspace(.generic) u8;")).toContain(
      '<span class="hljs-keyword">addrspace</span>',
    );
  });

  test("keywords removed from the language are not highlighted", () => {
    for (const stale of ["async", "await", "promise", "usingnamespace"]) {
      expect(render(`const x = ${stale};`)).not.toContain("hljs-keyword\">" + stale);
    }
  });

  test("current primitive types are highlighted", () => {
    expect(render("const p: ?*anyopaque = null;")).toContain(
      '<span class="hljs-type">anyopaque</span>',
    );
    expect(render("const c: c_char = 'a';")).toContain(
      '<span class="hljs-type">c_char</span>',
    );
    expect(render("const big: f80 = 1.0;")).toContain(
      '<span class="hljs-type">f80</span>',
    );
  });

  test("c_void is gone from the type list", () => {
    expect(render("const p: c_void = x;")).not.toContain(
      '<span class="hljs-type">c_void</span>',
    );
  });

  test("quoted identifiers stay one token", () => {
    const html = render('const q = @"weird name";');
    expect(html).toContain('<span class="hljs-title">@&quot;weird name&quot;</span>');
    expect(html).not.toContain("hljs-string");
  });

  test("builtins are highlighted", () => {
    expect(render('const std = @import("std");')).toContain(
      '<span class="hljs-built_in">@import</span>',
    );
    expect(render("const n = @intFromEnum(e);")).toContain(
      '<span class="hljs-built_in">@intFromEnum</span>',
    );
  });

  test("function declarations name the function", () => {
    expect(render("pub fn main() !void {}")).toContain(
      '<span class="hljs-title function_">main</span>',
    );
  });

  test("comments, doc comments and tags", () => {
    expect(render("//! module docs")).toContain(
      '<span class="hljs-comment">//! module docs</span>',
    );
    expect(render("/// TODO: refine")).toContain(
      '<span class="hljs-doctag">TODO:</span>',
    );
  });

  test("multiline strings are strings", () => {
    expect(render("const m =\n    \\\\line one\n;")).toContain(
      '<span class="hljs-string">\\\\line one</span>',
    );
  });

  test("a Zig 0.16 snippet highlights end to end", () => {
    const html = render(`const std = @import("std");

pub fn main() !void {
    var buf: [1024]u8 = undefined;
    var w = std.fs.File.stdout().writer(&buf);
    try w.interface.print("hi {d}\\n", .{@as(u32, 7)});
    try w.interface.flush();
}`);
    for (const cls of [
      "hljs-keyword",
      "hljs-built_in",
      "hljs-type",
      "hljs-number",
      "hljs-string",
      "hljs-literal",
      "hljs-title function_",
    ]) {
      expect(html).toContain(cls);
    }
  });

  test("auto-detection still picks Zig", () => {
    const detected = hljs.highlightAuto(
      'const std = @import("std");\npub fn main() !void {}',
    ).language;
    expect(detected).toBe("zig");
  });
});
