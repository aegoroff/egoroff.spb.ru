<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE xsl:stylesheet SYSTEM "dtd/entities.dtd">
<!-- $Id: apache_module.xsl,v 1.12 2004/02/06 15:53:10 egr Exp $ -->

<xsl:stylesheet
	version="1.0"
	xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
>

	<xsl:preserve-space elements="p"/>

  <!-- Constants used for case translation -->
  <xsl:variable name="lowercase" select="'abcdefghijklmnopqrstuvwxyz'" />
  <xsl:variable name="uppercase" select="'ABCDEFGHIJKLMNOPQRSTUVWXYZ'" />

  <!--
    space separated list of blockelements defined in common.dtd
    used for inline content catching in <example>s
  -->
  <xsl:variable name="blockelements">
    p  example  note  table  ul  ol  dl  pre  img  blockquote
  </xsl:variable>


	<xsl:template match="lastRevision">
		<xsl:call-template name="getDate"/>
	</xsl:template>

	<xsl:template name="getDate">
		<xsl:choose>
			<xsl:when test="@isUpdated = 'true'">
				<span style="color: red">
					<xsl:call-template name="hDayMonthGenitiveYear"/>
				</span>
			</xsl:when>
			<xsl:otherwise>
				<span>
					<xsl:call-template name="hDayMonthGenitiveYear"/>
				</span>
			</xsl:otherwise>
		</xsl:choose>
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
  <xsl:variable name="in-modulesynopsis"><xsl:choose>
    <xsl:when test="/quickreference">0</xsl:when>

    <xsl:otherwise>1</xsl:otherwise></xsl:choose>
  </xsl:variable>

  <!--                                                            -->
  <!-- Process a documentation section                            -->
  <!--                                                            -->
  <xsl:template match="section">

    <div class="section">

      <!-- Section heading -->

	  <h2>
        <xsl:call-template name="section_heading"/>
      </h2>

      <!-- Section body -->
      <xsl:apply-templates/>

    </div> <!-- /.section -->
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

          <xsl:text> </xsl:text>
          <a id="{$lowername}" name="{$lowername}">
            Директива
          </a>
        </h2>
        <!-- Directive header -->
        <div class="directive">
          <div class="tr">
            <div class="th">

                Описание:

            </div>
            <div class="td">
              <xsl:value-of select="description"/>
            </div>
          </div>

          <div class="tr">
            <div class="th">

                Синтаксис:

            </div>
            <div class="td">
              <xsl:apply-templates select="syntax"/>
            </div>
          </div>

          <xsl:if test="default">
          <div class="tr">
            <div class="th">

                Значение по умолчанию:

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

                Контекст:

            </div>
            <div class="td">
              <xsl:apply-templates select="contextlist"/>
            </div>
          </div>

          <xsl:if test="override">
          <div class="tr">
            <div class="th">

                Разрешение:

            </div>
            <div class="td">
              <xsl:value-of select="override"/>
            </div>
          </div>
          </xsl:if>

          <div class="tr">
            <div class="th">

                Статус:

            </div>
            <div class="td">
              <xsl:value-of select="../status"/>
            </div>
          </div>

          <div class="tr">
            <div class="th">

                Модуль:

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

                Совместимость:

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
            См. также
          </h3>

          <ul>
            <xsl:for-each select="seealso">
              <li>
                <xsl:apply-templates/>
              </li>
            </xsl:for-each>
          </ul>
        </xsl:if>

      </div> <!-- /directive-section -->
    </xsl:if>
  </xsl:template>
  <!-- /directivesynopsis -->

	<xsl:template name="page_columns">
		<div class="page-content">
			<xsl:apply-templates select="modulesynopsis/name"/>

              <h3>
                Резюме
              </h3>

			<xsl:apply-templates select="modulesynopsis/summary"/>

          <div id="quickview">

            <!-- Index of directives, automatically generated from
                 directivesynopsis/name -->
            <h3 class="directives">
              Директивы
            </h3>

            <xsl:if test="modulesynopsis/directivesynopsis">
              <ul id="toc">
                <xsl:for-each select="modulesynopsis/directivesynopsis">
                  <xsl:sort select="name"/>
                  <xsl:variable name="lowername" select="translate(name, $uppercase, $lowercase)"/>

                  <xsl:if test="not(@location)">
                    <li>
                      <a href="#{$lowername}">
                        <xsl:if test="@type='section'">&lt;</xsl:if>
                        <xsl:value-of select="name"/>
                        <xsl:if test="@type='section'">&gt;</xsl:if>
                      </a>
                    </li>
                  </xsl:if>

                  <xsl:if test="@location">
                    <xsl:variable name="lowerlocation" select="translate(@location, $uppercase, $lowercase)"/>

                    <li>
                      <a href="{$lowerlocation}.html#{$lowername}">
                        <xsl:if test="@type='section'">&lt;</xsl:if>
                        <xsl:value-of select="name"/>
                        <xsl:if test="@type='section'">&gt;</xsl:if>
                      </a>
                    </li>
                  </xsl:if>
                </xsl:for-each>
              </ul> <!-- /toc -->
            </xsl:if>

            <xsl:if test="not(modulesynopsis/directivesynopsis)">
              <p>
                Этот модуль не предоставляет никаких директив.
              </p>
            </xsl:if>

            <xsl:if test="modulesynopsis/section">
			  <h3>
                <xsl:value-of select="$messages/message[@name='topics']"/>
              </h3>
              <ul id="topics">
                Темы
              </ul>
            </xsl:if>

            <xsl:if test="modulesynopsis/seealso">
	      <h3>
                  См. также
              </h3>
            
              <ul class="seealso">
                <xsl:for-each select="modulesynopsis/seealso">
                  <li>
                    <xsl:apply-templates/>
                  </li>
                </xsl:for-each>
              </ul>
            </xsl:if>

          </div> <!-- /quickview -->

			<xsl:apply-templates select="modulesynopsis/section"/>

          <!-- Directive documentation -->
          <xsl:apply-templates select="modulesynopsis/directivesynopsis">
            <xsl:sort select="name"/>
          </xsl:apply-templates>

		</div>
	</xsl:template>

	<xsl:template match="name">
		<h1>
			Модуль Apache
			<xsl:text> </xsl:text> 
 			<xsl:value-of select="."/>
		</h1>
	</xsl:template>

	<xsl:template match="example">
		<div><p><code><xsl:apply-templates/></code></p></div>
	</xsl:template>

	<xsl:template match="dfn">
		<dfn style="font-weight: bold"><xsl:apply-templates/></dfn>
	</xsl:template>

	<xsl:template match="pre">
		<pre class="code"><xsl:apply-templates/></pre>
	</xsl:template>

	<xsl:template match="blockquote">
		<blockquote>
		<xsl:apply-templates/>
		</blockquote>
	</xsl:template>

	<xsl:template match="img">
		<img><xsl:copy-of select="@*"/></img>
	</xsl:template>

  <!--                                                    -->
  <!-- <example>                                          -->
  <!-- iterate over *all* nodes; bare text and other      -->
  <!-- inline stuff is wrapped into <p><code>, block      -->
  <!-- level elements (defined in $blockelements) are     -->
  <!-- applied "as is"                                    -->
  <!--                                                    -->
  <xsl:variable name="blocks"
              select="concat(' ', normalize-space($blockelements), ' ')"/>

  <xsl:template match="example">
    <div class="example">
       <xsl:apply-templates select="title" mode="print"/>

       <xsl:for-each select="./node()">
         <xsl:variable name="is-block-node" select="boolean(contains($blocks, concat(' ',local-name(),' ')))"/>
         <xsl:variable name="bb" select="count(preceding-sibling::*[contains($blocks,concat(' ',local-name(),' '))])"/>

         <xsl:if test="$is-block-node or position()=last()">
           <xsl:variable name="content">
             <xsl:apply-templates select="preceding-sibling::node()[count(preceding-sibling::*[contains($blocks,concat(' ',local-name(),' '))]) &gt;= $bb]"/>
             <xsl:apply-templates select="self::node()[not($is-block-node)]"/>
           </xsl:variable>

           <!-- apply bare text only, if it's not only \s or empty -->
           <xsl:if test="normalize-space($content) != ''">
             <p><code>
               <xsl:copy-of select="$content"/>
             </code></p>
           </xsl:if>

           <xsl:apply-templates select="self::node()[$is-block-node]"/>
         </xsl:if>

       </xsl:for-each>
       <!-- /node() -->

     </div> <!-- /.example -->
  </xsl:template>
  <!-- /example -->

</xsl:stylesheet>