import unittest

from markdown import Markdown
from pygments.lexers import get_lexer_by_name
from swmm_pygments import SwmmLexer


SAMPLE = """[RAINGAGES]
RG1 VOLUME 0:05 1.0 TIMESERIES RAIN_TS

[AMM_MODELS]
;;Name RainGage ColdTemp HotTemp
; ordinary comment
SAN1 RG1 30.0 70.0

[AMM_COMPONENTS]
SAN1 FAST STANDARD 0.010 0.0 2.0
SAN1 BASE BASEFLOW 240.0 720.0 240.0
"""


class SwmmHighlightTest(unittest.TestCase):
    def test_markdown_extension_highlights_swmm_input(self):
        markdown = Markdown(
            extensions=["swmm_pygments", "pymdownx.superfences"],
            extension_configs={
                "swmm_pygments": {"pygments_lang_class": True}
            },
        )
        html = markdown.convert(f"```swmm\n{SAMPLE}```")
        expected = (
            'class="language-swmm highlight"',
            '<span class="kn">[AMM_MODELS]</span>',
            '<span class="gh">;;Name RainGage ColdTemp HotTemp</span>',
            '<span class="c1">; ordinary comment</span>',
            '<span class="k">VOLUME</span>',
            '<span class="k">TIMESERIES</span>',
            '<span class="k">STANDARD</span>',
            '<span class="k">BASEFLOW</span>',
            '<span class="m">0:05</span>',
            '<span class="m">30.0</span>',
            '<span class="n">SAN1</span>',
        )

        for fragment in expected:
            with self.subTest(fragment=fragment):
                self.assertIn(fragment, html)

    def test_pygments_entry_point(self):
        self.assertIsInstance(get_lexer_by_name("swmm"), SwmmLexer)


if __name__ == "__main__":
    unittest.main()
