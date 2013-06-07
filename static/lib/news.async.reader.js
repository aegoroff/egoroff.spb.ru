$(function() {
    "use strict";
    var more = $("button#moreNews");
    more.click(function () {
        var ov = $("input#offsetValue");
        var lv = $("input#limitValue");
        var offset = ov.val();
        var limit = lv.val();
        $.get('/recent.atom?offset=' + offset + '&limit=' + limit, onRssSuccess);
        ov.attr("value", parseInt(offset) + parseInt(limit));
    });
    more.button();

    var month = $('div#accordion > div > ul > li > a');
    month.click(function (event) {
        var href = event.target.hash;
        var vars = href.split('&');
        var y = 1;
        var m = 1;
        for (var i = 0; i < vars.length; i++) {
            var pair = vars[i].split('=');
            if (pair[0] == 'month') {
                m = pair[1];
            } else{
                y = pair[1];
            }
        }
        var dlLog = $("body").find("dl#blog");
        dlLog.empty();
        dlLog.append("<dt>Загрузка данных. Пожалуйста подождите ...</dt>");
        $.get('/recent.atom?year=' + y + '&month=' + m, onArchieveRssSuccess);
    });
    month.button();
});

function onRssSuccess(xml) {
    "use strict";
    var items = $("entry", xml);
    var dlLog = $("body").find("dl#log");
    loadBlog(items, xml, dlLog);
    var lv = $("input#limitValue");
    var limit = lv.val();
    if (items.length < parseInt(limit)) {
        $("button#moreNews").remove();
    }
}

function onArchieveRssSuccess(xml) {
    "use strict";
    var items = $("entry", xml);
    var dlLog = $("body").find("dl#blog");
    dlLog.empty();
    loadBlog(items, xml, dlLog);

    var limit = 20;
    if (items.length < limit) {
        $("ul.pager").remove();
        $("div.pagination").remove();
    }
}

function loadBlog(items, xml, dlLog) {
    items.each(
        function (item) {
            var dt = $("entry published", xml).get(item);
            var title = $("entry title", xml).get(item);
            var description = $("entry content", xml).get(item);
            var date = dt.textContent == undefined ? dt.text : dt.textContent;
            var humanReadableDate = $.format.date(date, "dd MMMM yyyy");
            var d = description.textContent == undefined ? description.text : description.textContent;
            var t = title.textContent == undefined ? title.text : title.textContent;

            var link = "<a href=\"" + $("entry link", xml).get(item).getAttribute("href") + "\">" + t + "</a>";
            dlLog.append("<dt><small><i class=\"icon-calendar\"></i> <span class=\"shortDateFormat\">" + humanReadableDate + "</span></small>&nbsp;" + link + "</dt>");
            dlLog.append("<dd>" + d + "</dd>");
        }
    );
}