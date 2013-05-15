<?xml version="1.0" encoding="UTF-8"?>
<!-- $Id: apache_common_elem.xsl,v 1.8 2005/09/10 08:31:40 egr Exp $ -->

<xsl:stylesheet
        version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
        >

    <xsl:variable name="message_file">messages.xml</xsl:variable>

    <!-- Read the localized messages from the specified language file -->
    <xsl:variable name="messages" select="document($message_file)/messages"/>


    <xsl:preserve-space elements="p"/>

    <!-- Constants used for case translation -->
    <xsl:variable name="lowercase" select="'abcdefghijklmnopqrstuvwxyz'"/>
    <xsl:variable name="uppercase" select="'ABCDEFGHIJKLMNOPQRSTUVWXYZ'"/>

    <!--
      space separated list of blockelements defined in common.dtd
      used for inline content catching in <example>s
    -->
    <xsl:variable name="blockelements">
        p example note table ul ol dl pre img blockquote
    </xsl:variable>


    <xsl:template match="header">
        <!-- Дата публикации -->
        <div class="published">
            <strong>
                <xsl:value-of select="$messages/message[@name='last_edition']"/>
                <xsl:text>:</xsl:text>
            </strong>

            <span id="plusone-div"></span>
        </div>
    </xsl:template>

    <!-- relative path to /manual/ -->
    <xsl:variable name="path">
        <xsl:choose>
            <xsl:when test="*/relativepath/@href">
                <xsl:value-of select="*/relativepath/@href"/>
            </xsl:when>
            <xsl:otherwise>
                <xsl:value-of select="'..'"/>
            </xsl:otherwise>
        </xsl:choose>
    </xsl:variable>

    <!-- make sure, we set relative anchors
         only, if we're actually transforming
         a modulefile (see <directive>) -->
    <xsl:variable name="in-modulesynopsis">
        <xsl:choose>
            <xsl:when test="/quickreference">0</xsl:when>

            <xsl:otherwise>1</xsl:otherwise>
        </xsl:choose>
    </xsl:variable>

    <xsl:template name="site_style">
    </xsl:template>

    <!--                                                            -->
    <!-- Process a documentation section                            -->
    <!--                                                            -->
    <xsl:template match="section">

        <div class="section">

            <h2>
                <xsl:call-template name="section_heading"/>
            </h2>

            <!-- Section body -->
            <xsl:apply-templates/>

        </div>
        <!-- /.section -->
    </xsl:template>
    <!-- /section -->

    <!--                                                            -->
    <!-- handle subsections (lower level headings)                  -->
    <!--                                                            -->
    <xsl:template match="section/section">

        <!-- Section heading -->
        <h3>
            <xsl:call-template name="section_heading"/>
        </h3>

        <!-- Section body -->
        <xsl:apply-templates/>

    </xsl:template>
    <!-- /section/section -->

    <!--                                                            -->
    <!-- handle subsubsections (h4)                                 -->
    <!--                                                            -->
    <xsl:template match="section/section/section">

        <!-- Section heading -->
        <h4>
            <xsl:call-template name="section_heading"/>
        </h4>

        <!-- Section body -->
        <xsl:apply-templates/>

    </xsl:template>
    <!-- /section/section/section -->

    <!-- Заголовок раздела, общая часть для всех разделов -->
    <!-- мудаки из ASF не додумались общность вынести в отд. шаблон -->

    <xsl:template name="section_heading">
        <xsl:if test="@id">
            <a id="{@id}" name="{@id}">
                <xsl:apply-templates select="title" mode="print"/>
            </a>
        </xsl:if>

        <xsl:if test="not(@id)">
            <xsl:apply-templates select="title" mode="print"/>
        </xsl:if>
    </xsl:template>

    <!--                                                            -->
    <!-- section nesting > h4 is not supported for now              -->
    <!--                                                            -->
    <xsl:template match="section/section/section/section">
        <xsl:message terminate="yes">
            <xsl:text>FATAL: exceeding maximum section nesting level.

                Perhaps you should consider to split your document into
                several ones...
            </xsl:text>
        </xsl:message>
    </xsl:template>
    <!-- /section/section/section/section -->

    <!--                                                            -->
    <!-- (sub)section titles                                        -->
    <!--                                                            -->
    <xsl:template match="section/title" mode="print">
        <xsl:apply-templates/>
    </xsl:template>

    <xsl:template match="section/title">
        <!-- Don't print the title twice -->
    </xsl:template>
    <!-- /section/title -->


    <!--                                                            -->
    <!-- generate section index                                     -->
    <!--                                                            -->
    <xsl:template match="section" mode="index">

        <xsl:if test="@id">
            <li>
                <a href="#{@id}">
                    <xsl:apply-templates select="title" mode="print"/>
                </a>
            </li>
        </xsl:if>

        <xsl:if test="not(@id)">
            <li>
                <xsl:apply-templates select="title" mode="print"/>

                <!-- nested sections -->
                <!-- NOT FOR NOW     -->
                <!--
                <xsl:if test="section">
                  <ul>
                    <xsl:apply-templates select="section" mode="index"/>
                  </ul>
                </xsl:if>
                -->
            </li>
        </xsl:if>

    </xsl:template>
    <!-- /section index -->

    <!--                                                    -->
    <!-- example/title                                      -->
    <!--                                                    -->
    <xsl:template match="example/title" mode="print">
        <h3>
            <xsl:apply-templates/>
        </h3>
    </xsl:template>

    <xsl:template match="example/title">
        <!-- don't print twice -->
    </xsl:template>
    <!-- /example/title -->

    <!--                                                    -->
    <!-- <note>                                             -->
    <!-- Notes are placed in a table. Uses different back-  -->
    <!-- ground colors, depending on type of note.          -->
    <!--                                                    -->
    <xsl:template match="note">
        <xsl:choose>
            <xsl:when test="@type='warning'">
                <div class="warning">
                    <xsl:apply-templates/>
                </div>
            </xsl:when>

            <xsl:otherwise>
                <div class="note">
                    <xsl:apply-templates/>
                </div>
            </xsl:otherwise>
        </xsl:choose>
    </xsl:template>
    <!-- /note -->

    <!--                                                    -->
    <!-- <note><title>                                      -->
    <!--                                                    -->
    <xsl:template match="note/title">
        <h3>
            <xsl:apply-templates/>
        </h3>
    </xsl:template>
    <!-- /note/title -->

    <!--                                                    -->
    <!-- <directive>                                        -->
    <!-- Inserts link to another directive, which might be  -->
    <!-- in another module. References are converted into   -->
    <!-- lower case.                                        -->
    <!--                                                    -->
    <xsl:template match="directive" name="directive">
        <code class="directive">

            <xsl:if test="@module">
                <xsl:variable name="lowerdirective" select="translate(., $uppercase, $lowercase)"/>

                <xsl:choose>
                    <xsl:when test="$in-modulesynopsis = '1' and @module = /modulesynopsis/name">
                        <a href="#{$lowerdirective}">
                            <xsl:if test="@type='section'">&lt;</xsl:if>
                            <xsl:value-of select="."/>
                            <xsl:if test="@type='section'">&gt;</xsl:if>
                        </a>
                    </xsl:when>

                    <xsl:otherwise>
                        <xsl:variable name="obs_">
                            <xsl:if test="@status = 'obsolete'">obs_</xsl:if>
                        </xsl:variable>

                        <a href="{$path}/apache/{$obs_}{@module}.html#{$lowerdirective}">
                            <xsl:if test="@type='section'">&lt;</xsl:if>
                            <xsl:value-of select="."/>
                            <xsl:if test="@type='section'">&gt;</xsl:if>
                        </a>
                    </xsl:otherwise>
                </xsl:choose>
            </xsl:if>

            <xsl:if test="not(@module)">
                <xsl:if test="@type='section'">&lt;</xsl:if>
                <xsl:value-of select="."/>
                <xsl:if test="@type='section'">&gt;</xsl:if>
            </xsl:if>

        </code>
    </xsl:template>
    <!-- /directive -->

    <!--                                                    -->
    <!-- <module>                                           -->
    <!-- Inserts a link to refereed module                  -->
    <!--                                                    -->
    <xsl:template match="module" name="module">
        <xsl:variable name="obs_">
            <xsl:if test="@status = 'obsolete'">obs_</xsl:if>
        </xsl:variable>

        <code class="module">
            <a href="{$path}/apache/{$obs_}{.}.html">
                <xsl:value-of select="."/>
            </a>
        </code>
    </xsl:template>
    <!-- /module -->

    <!--                                                            -->
    <!--    Directivesynopsis                                       -->
    <!--                                                            -->
    <xsl:template match="directivesynopsis">
        <xsl:if test="not(@location)">

            <div class="directive-section">
                <xsl:variable name="lowername" select="translate(name, $uppercase, $lowercase)"/>
                <!-- Directive heading gets both mixed case and lowercase anchors,
                     and includes lt/gt only for "section" directives -->
                <h2>

                    <xsl:if test="@type='section'">&lt;</xsl:if>
                    <xsl:value-of select="name"/>
                    <xsl:if test="@type='section'">&gt;</xsl:if>

                    <xsl:text></xsl:text>
                    <a id="{$lowername}" name="{$lowername}">
                        <xsl:value-of select="$messages/message[@name='directive']"/>
                    </a>
                </h2>
                <!-- Directive header -->
                <div class="directive">
                    <div class="tr">
                        <div class="th">

                            <xsl:value-of select="$messages/message[@name='description']"/>:

                        </div>
                        <div class="td">
                            <xsl:value-of select="description"/>
                        </div>
                    </div>

                    <div class="tr">
                        <div class="th">

                            <xsl:value-of select="$messages/message[@name='syntax']"/>:

                        </div>
                        <div class="td">
                            <xsl:apply-templates select="syntax"/>
                        </div>
                    </div>

                    <xsl:if test="default">
                        <div class="tr">
                            <div class="th">

                                <xsl:value-of select="$messages/message[@name='default']"/>:

                            </div>
                            <div class="td">
                                <code>
                                    <xsl:value-of select="default"/>
                                </code>
                            </div>
                        </div>
                    </xsl:if>

                    <div class="tr">
                        <div class="th">

                            <xsl:value-of select="$messages/message[@name='context']"/>:

                        </div>
                        <div class="td">
                            <xsl:apply-templates select="contextlist"/>
                        </div>
                    </div>

                    <xsl:if test="override">
                        <div class="tr">
                            <div class="th">

                                <xsl:value-of select="$messages/message[@name='override']"/>:

                            </div>
                            <div class="td">
                                <xsl:value-of select="override"/>
                            </div>
                        </div>
                    </xsl:if>

                    <div class="tr">
                        <div class="th">

                            <xsl:value-of select="$messages/message[@name='status']"/>:

                        </div>
                        <div class="td">
                            <xsl:value-of select="../status"/>
                        </div>
                    </div>

                    <div class="tr">
                        <div class="th">

                            <xsl:value-of select="$messages/message[@name='module']"/>:

                        </div>
                        <div class="td">
                            <xsl:if test="modulelist">
                                <xsl:apply-templates select="modulelist"/>
                            </xsl:if>

                            <xsl:if test="not(modulelist)">
                                <xsl:value-of select="../name"/>
                            </xsl:if>
                        </div>
                    </div>

                    <xsl:if test="compatibility">
                        <div class="tr">
                            <div class="th">

                                <xsl:value-of select="$messages/message[@name='compatibility']"/>:

                            </div>
                            <div class="td">
                                <xsl:value-of select="compatibility"/>
                            </div>
                        </div>
                    </xsl:if>
                </div>

                <xsl:apply-templates select="usage"/>

                <xsl:if test="seealso">
                    <h3>
                        <xsl:value-of select="$messages/message[@name='seealso']"/>
                    </h3>

                    <ul>
                        <xsl:for-each select="seealso">
                            <li>
                                <xsl:apply-templates/>
                            </li>
                        </xsl:for-each>
                    </ul>
                </xsl:if>

            </div>
            <!-- /directive-section -->
        </xsl:if>
    </xsl:template>
    <!-- /directivesynopsis -->

	<xsl:template match="br">
		<br/>
	</xsl:template>

	<xsl:template match="center">
		<div style="text-align: center;"><xsl:apply-templates/></div>
	</xsl:template>

    <xsl:template match="em">
        <em>
            <xsl:apply-templates/>
        </em>
    </xsl:template>

    <xsl:template match="strong">
        <strong>
            <xsl:apply-templates/>
        </strong>
    </xsl:template>

    <xsl:template match="ol">
        <ol>
            <xsl:apply-templates/>
        </ol>
    </xsl:template>

    <xsl:template match="ul">
        <ul>
            <xsl:apply-templates/>
        </ul>
    </xsl:template>

    <xsl:template match="li">
        <li>
            <xsl:apply-templates/>
        </li>
    </xsl:template>


    <xsl:template match="dl">
        <dl>
            <xsl:apply-templates/>
        </dl>
    </xsl:template>

    <xsl:template match="dt">
        <dt>
            <xsl:apply-templates/>
        </dt>
    </xsl:template>

    <xsl:template match="dd">
        <dd>
            <xsl:apply-templates/>
        </dd>
    </xsl:template>

    <xsl:template match="code">
        <code>
            <xsl:apply-templates/>
        </code>
    </xsl:template>


	<xsl:template match="sub">
		<sub><xsl:apply-templates/></sub>
	</xsl:template>

	<xsl:template match="sup">
		<sup><xsl:apply-templates/></sup>
	</xsl:template>


	<xsl:template match="acronym">
		<acronym>
			<xsl:copy-of select="@*"/>
			<xsl:apply-templates/>
		</acronym>
	</xsl:template>

	<xsl:template match="p">
		<p><xsl:apply-templates/></p>
	</xsl:template>

    <xsl:template match="h2">
    	<h2><xsl:apply-templates/></h2>
    </xsl:template>

    <xsl:template match="h3">
      	<h3><xsl:apply-templates/></h3>
    </xsl:template>

    <xsl:template match="h4">
        <h4><xsl:apply-templates/></h4>
    </xsl:template>

    <xsl:template match="a">
        <a href="{@href}">
            <xsl:if test="@title">
                <xsl:attribute name="title">
                    <xsl:value-of select="@title"/>
                </xsl:attribute>
            </xsl:if>
            <xsl:apply-templates/>
        </a>
    </xsl:template>


    	<!-- Строчные цитаты -->

	<xsl:template match="q">
		<q>
			<xsl:if test="@cite">
				<xsl:attribute name="cite">
					<xsl:value-of select="@cite"/>
				</xsl:attribute>
			</xsl:if>
		<xsl:apply-templates/>
		</q>
	</xsl:template>

	<!-- Блочные цитаты -->

	<xsl:template match="quote">
		<blockquote>
			<xsl:if test="@cite">
				<xsl:attribute name="cite">
					<xsl:value-of select="@cite"/>
				</xsl:attribute>
			</xsl:if>
		<xsl:apply-templates/>
		</blockquote>
	</xsl:template>


</xsl:stylesheet>
