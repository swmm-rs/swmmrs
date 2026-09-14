function initializeCollapsibleToc() {
  document
    .querySelectorAll('[data-md-component="toc"] > .md-nav__item')
    .forEach((item, index) => {
      const children = item.querySelector(":scope > .md-nav")
      const link = item.querySelector(":scope > .md-nav__link")

      if (!children || !link || item.dataset.collapsibleToc) return

      item.dataset.collapsibleToc = "true"
      children.id ||= `toc-section-${index}`
      children.hidden = true

      const toggle = document.createElement("button")
      toggle.type = "button"
      toggle.className = "toc-toggle"
      toggle.setAttribute("aria-controls", children.id)
      toggle.setAttribute("aria-expanded", "false")
      toggle.setAttribute(
        "aria-label",
        `Expand ${link.textContent.trim()} section`,
      )
      toggle.addEventListener("click", () => {
        const expanded = toggle.getAttribute("aria-expanded") === "true"
        toggle.setAttribute("aria-expanded", String(!expanded))
        toggle.setAttribute(
          "aria-label",
          `${expanded ? "Expand" : "Collapse"} ${link.textContent.trim()} section`,
        )
        children.hidden = expanded
      })

      link.after(toggle)
    })
}

if (typeof document$ !== "undefined") {
  document$.subscribe(initializeCollapsibleToc)
} else {
  document.addEventListener("DOMContentLoaded", initializeCollapsibleToc)
}
