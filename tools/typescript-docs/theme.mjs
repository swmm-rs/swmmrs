import { TypeContext } from "typedoc";
import { MarkdownTheme } from "typedoc-plugin-markdown";

class ReferenceTheme extends MarkdownTheme {
  getRenderContext(page) {
    const context = super.getRenderContext(page);
    // typedoc-plugin-markdown 4.13 only parenthesizes union array elements.
    // Preserve TypeScript precedence for readonly matrices as well, using
    // TypeDoc's own rules rather than rewriting generated Markdown.
    context.partials.arrayType = model => {
      const element = context.partials.someType(model.elementType);
      return model.elementType.needsParenthesis(TypeContext.arrayElement)
        ? `(${element})[]`
        : `${element}[]`;
    };
    // The constructor partial bypasses memberContainer and drops custom anchors.
    // Emit the router's anchor on its heading so inherited constructor links
    // cannot collide with Markdown's independently numbered "Constructor" IDs.
    context.partials.constructor = (model, options) => {
      const anchor = context.router.getAnchor(model);
      const title = `${"#".repeat(options.headingLevel)} Constructor {#${anchor}}`;
      const signatures = model.signatures?.map(signature => context.partials.signature(signature, {
        headingLevel: options.headingLevel + 1,
      })) ?? [];
      return [title, ...signatures].join("\n\n");
    };
    return context;
  }
}

export function load(app) {
  app.renderer.defineTheme("swmmrs-reference", ReferenceTheme);
}
