import re
import typing

from pygments.lexer import RegexLexer, words
from pygments.token import (
    Comment,
    Generic,
    Keyword,
    Name,
    Number,
    String,
    Text,
    Whitespace,
)
from pymdownx.highlight import Highlight, HighlightExtension


class SwmmLexer(RegexLexer):
    name = "SWMM"
    aliases = ["swmm", "inp"]
    filenames = ["*.inp"]
    mimetypes = ["text/x-swmm"]

    flags = re.IGNORECASE | re.MULTILINE

    shapes = (
        "CIRCULAR",
        "FORCE_MAIN",
        "FILLED_CIRCULAR",
        "RECT_CLOSED",
        "RECT_OPEN",
        "TRAPEZOIDAL",
        "TRIANGULAR",
        "HORIZ_ELLIPSE",
        "VERT_ELLIPSE",
        "ARCH",
        "PARABOLIC",
        "POWER",
        "RECT_TRIANGULAR",
        "RECT_ROUND",
        "MODBASKETHANDLE",
        "EGG",
        "HORSESHOE",
        "GOTHIC",
        "CATENARY",
        "SEMIELLIPTICAL",
        "BASKETHANDLE",
        "SEMICIRCULAR",
    )

    keywords = (
        "USE",
        "SAVE",
        "TIMESERIES",
        "FILE",
        "INTENSITY",
        "VOLUME",
        "CUMULATIVE",
        "IMPERVIOUS",
        "PERVIOUS",
        "SURFACE",
        "SOIL",
        "PAVEMENT",
        "STORAGE",
        "DRAIN",
        "DRAINMAT",
        "REMOVALS",
        "BC",
        "RG",
        "GR",
        "IT",
        "PP",
        "RB",
        "RD",
        "VS",
        "LATERAL",
        "DEEP",
        "PLOWABLE",
        "REMOVAL",
        "FREE",
        "NORMAL",
        "FIXED",
        "TIDAL",
        "OVERFLOW",
        "CUTOFF",
        "TABULAR",
        "WEIR",
        "FUNCTIONAL",
        "SIDE",
        "BOTTOM",
        "YES",
        "NO",
        "TRANSVERSE",
        "SIDEFLOW",
        "V-NOTCH",
        "ROADWAY",
        "OUTLET",
        "HOURLY",
        "MONTHLY",
        "WEEKLY",
        "WEEKEND",
        "POW",
        "EXP",
        "SAT",
        "EXT",
        "RC",
        "EMC",
        "FLOW",
        "DEPTH",
        "AREA",
        "DT",
        "HRT",
        "CONCEN",
        "MASS",
        "SHORT",
        "MEDIUM",
        "LONG",
        "SHAPE",
        "DIVERSION",
        "PUMP1",
        "PUMP2",
        "PUMP3",
        "PUMP4",
        "PUMP5",
        "RATING",
        "CONTROL",
        "FEET",
        "METERS",
        "DEGREES",
        "NONE",
        "MGD",
        "CFS",
        "GPM",
        "LPS",
        "CMS",
        "MLD",
        "HORTON",
        "MODIFIED_GREEN_AMPT",
        "GREEN_AMPT",
        "CURVE_NUMBER",
        "STEADY",
        "KINWAVE",
        "DYNWAVE",
        "ELEVATION",
        "H-W",
        "D-W",
        "PARTIAL",
        "FULL",
        "SLOPE",
        "FROUDE",
        "BOTH",
        "ALL",
        "INPUT",
        "CONTINUITY",
        "FLOWSTATS",
        "CONTROLS",
        "SUBCATCHMENTS",
        "NODES",
        "LINKS",
        "CONSTANT",
        "TEMPERATURE",
        "RECOVERY",
        "DRY_ONLY",
        "WINDSPEED",
        "SNOWMELT",
        "EVAPORATION",
        "RAINFALL",
        "CONDUCTIVITY",
        "FLOW_UNITS",
        "INFILTRATION",
        "FLOW_ROUTING",
        "LINK_OFFSETS",
        "FORCE_MAIN_EQUATION",
        "IGNORE_RAINFALL",
        "IGNORE_SNOWMELT",
        "IGNORE_GROUNDWATER",
        "IGNORE_RDII",
        "IGNORE_ROUTING",
        "IGNORE_QUALITY",
        "ALLOW_PONDING",
        "SKIP_STEADY_STATE",
        "SYS_FLOW_TOL",
        "LAT_FLOW_TOL",
        "START_DATE",
        "START_TIME",
        "END_DATE",
        "END_TIME",
        "REPORT_START_DATE",
        "REPORT_START_TIME",
        "SWEEP_START",
        "SWEEP_END",
        "DRY_DAYS",
        "REPORT_STEP",
        "WET_STEP",
        "DRY_STEP",
        "ROUTING_STEP",
        "LENGTHENING_STEP",
        "VARIABLE_STEP",
        "MINIMUM_STEP",
        "RULE_STEP",
        "INERTIAL_DAMPING",
        "NORMAL_FLOW_LIMITED",
        "MIN_SURFAREA",
        "MIN_SLOPE",
        "MAX_TRIALS",
        "HEAD_TOLERANCE",
        "THREADS",
        "CUSTOM_ELLIPSE_MODEL",
        "TEMPDIR",
        "LID",
        "DIMENSIONS",
        "AVERAGES",
        "RDII",
        "FAST",
        "BASE",
        "STANDARD",
        "BASEFLOW",
    )

    tokens: typing.ClassVar = {
        "root": [
            # Double-quoted strings, including escaped characters.
            (r'"(?:\\.|[^"\\])*"', String.Double),
            # Single-quoted strings, including escaped characters.
            (r"'(?:\\.|[^'\\])*'", String.Single),
            # Table-header comments beginning with two semicolons.
            (r";;[^\n]*", String.Single),
            # Ordinary comments beginning with one semicolon.
            (r";[^\n]*", Comment.Single),
            # Section headers such as "[RAINGAGES]", optionally indented.
            (r"^\s*\[[^\]\n]+\]", Generic.Heading),
            # Known geometry names matched as complete words.
            (words(shapes, prefix=r"\b", suffix=r"\b"), Keyword.Type),
            # Known SWMM keywords matched as complete words.
            (words(keywords, prefix=r"\b", suffix=r"\b"), Keyword),
            # Times in HH:MM or HH:MM:SS, with optional fractional seconds.
            (r"\b\d{1,2}:\d{2}(?::\d{2}(?:\.\d+)?)?\b", Number),
            # Slash-separated dates with two- or four-digit years.
            (r"\b\d{1,2}/\d{1,2}/\d{2,4}\b", Number),
            # Signed integers, decimals, or lowercase scientific notation.
            (
                r"(?<![\w.])[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?(?![\w.])",
                Number,
            ),
            # Runs of whitespace.
            (r"\s+", Whitespace),
            # Remaining field text, stopping at whitespace or a semicolon.
            (r"[^\s;]+", Name),
            # Any unmatched character.
            (r".", Text),
        ]
    }


class SwmmHighlight(Highlight):
    def get_lexer(self, src, language, inline, stripnl):
        if language and language.lower() in SwmmLexer.aliases:
            return SwmmLexer(stripnl=stripnl), "swmm"
        return super().get_lexer(src, language, inline, stripnl)


class SwmmHighlightExtension(HighlightExtension):
    def get_pymdownx_highlighter(self):
        return SwmmHighlight


def makeExtension(*args, **kwargs):
    return SwmmHighlightExtension(*args, **kwargs)
