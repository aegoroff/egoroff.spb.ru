/**
 * SyntaxHighlighter
 * http://alexgorbatchev.com/SyntaxHighlighter
 *
 * SyntaxHighlighter is donationware. If you are using it, please donate.
 * http://alexgorbatchev.com/SyntaxHighlighter/donate.html
 *
 * @version
 * 1.0.0 (September 25 2010)
 * 
 * @copyright
 * Copyright (C) 2010 Alexander Egorov.
 *
 * @license
 * Dual licensed under the MIT and GPL licenses.
 */
;(function()
{
	// CommonJS
	typeof(require) != 'undefined' ? SyntaxHighlighter = require('shCore').SyntaxHighlighter : null;

    function process(match, regexInfo)
    {
        var constructor = SyntaxHighlighter.Match,
            code = match[0],
            tag = new XRegExp('(&lt;|<)[\\s\\/\\?]*(?<name>[:\\w-\\.]+)', 'xg').exec(code),
            result = []
            ;

        if (match.attributes != null)
        {
            var attributes,
                regex = new XRegExp('(?<name> [\\w:\\-\\.]+)' +
                                    '\\s*=\\s*' +
                                    '(?<value> ".*?"|\'.*?\'|\\w+)',
                                    'xg');

            while ((attributes = regex.exec(code)) != null)
            {
                result.push(new constructor(attributes.name, match.index + attributes.index, 'color1'));
            }
        }

        if (tag != null)
            result.push(
                new constructor(tag.name, match.index + tag[0].indexOf(tag.name), 'decorator')
            );

        return result;
    }

    function Brush()
	{
		var funcs	=	'auto eval connect cache process rem taint untaint ' +
                        '.roll .sql-string .unix-timestamp .last-day ' +
                        '.gmt-string .int .double .bool .inc .dec .mul .div .mod ' +
                        '.format apply-taint ' +
                        'bool ';

		var keywords =	'case CLASS -f -d in def USE for while break continue goto if is ne eq in switch throw try BASE use OPTIONS ' +
                        'GET_ SET_ ';


		this.regexList = [
			{ regex: new RegExp('^#.*$', 'gm'),                            css: 'comments' },			// one line comments
			{ regex: /(rem\{)([^\1])*?\}/gm,                               css: 'comments' },			// multiline comments
            { regex: /^@(?:BASE|USE|CLASS|OPTIONS)/gm, 					   css: 'keyword' },            // Class
            { regex: /^@[\w\-]+\[[\w^;\-]*\](?:\[[\w^;\-]*\])?(?:.*)$/gm,  css: 'color3' },            // function definition
            { regex: /(response|request|xdoc|string|status):\w+/g,		   css: 'functions' },         // system functions
            { regex: /(math|inet|file|date|void|form|cookie):\w+/g,		   css: 'functions' },         // system functions
            { regex: /(double|int|console|mail|json|sleep):\w+/g,		   css: 'functions' },         // system functions
            { regex: /(reflection|regex|memory|curl):\w+/g,		           css: 'functions' },         // system functions
            { regex: /\$\.(\w+)/g,		                                   css: 'color1' },            // hash key names
			{ regex: /\w+::\w+/g, 										   css: 'string' },            // constructors
            { regex: /\$\w+/g,											   css: 'variable' },			// variables
			{ regex: new RegExp(this.getKeywords(funcs), 'gmi'),		   css: 'functions' },			// common functions
			{ regex: new RegExp(this.getKeywords(keywords), 'gm'),		   css: 'keyword' },			// keyword
            { regex: new XRegExp('(&lt;|<)[\\s\\/\\?]*(\\w+)(?<attributes>.*?)[\\s\\/\\?]*(&gt;|>)', 'sg'), func: process }
			];

		this.forHtmlScript(SyntaxHighlighter.regexLib.phpScriptTags);
	};

	Brush.prototype	= new SyntaxHighlighter.Highlighter();
	Brush.aliases	= ['parser', 'php'];

	SyntaxHighlighter.brushes.Parser = Brush;

	// CommonJS
	typeof(exports) != 'undefined' ? exports.Brush = Brush : null;
})();
