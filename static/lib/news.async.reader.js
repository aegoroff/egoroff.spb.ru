$(function() {
    var more = $("button#moreNews");
    more.click(function () {
        var ov = $("input#offsetValue");
        var offset = ov.val();
        $.get('/recent.atom?offset=' + offset, onRssSuccess);
        ov.attr("value", parseInt(offset) + 20);
    });
    more.button();
});

function onRssSuccess(xml) {
    var items = $("entry", xml);
    items.each(
        function(item) {
            var dt = $("entry published", xml).get(item);
            var title = $("entry title", xml).get(item);
            var description = $("entry content", xml).get(item);
            var date = dt.textContent == undefined ? dt.text : dt.textContent;
            var humanReadableDate = $.format.date(date, "dd MMMM yyyy");
            var d = description.textContent == undefined ? description.text : description.textContent;
            var t = title.textContent == undefined ? title.text : title.textContent;
            var dlLog = $("body").find("dl#log");
            var link = "<a href=\"" + $("entry link", xml).get(item).getAttribute("href") + "\">" + t + "</a>"
            dlLog.append("<dt><dt><small><i class=\"icon-calendar\"></i> <span class=\"shortDateFormat\">" + humanReadableDate + "</span></small>&nbsp;" + link + "</dt>");
            dlLog.append("<dd>" + d + "</dd>");
        }
    );
    if (items.length < 20) {
        $("button#moreNews").remove()
    }
}