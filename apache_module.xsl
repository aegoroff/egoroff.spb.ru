<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE xsl:stylesheet SYSTEM "common/entities.dtd">
<!-- $Id: apache_module.xsl,v 1.12 2004/02/06 15:53:10 egr Exp $ -->

<xsl:stylesheet
	version="1.0"
	xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
>

	<xsl:import href="apache_common_elem.xsl"/>

	<xsl:template name="page_columns">
		<div class="page-content">
			<xsl:apply-templates select="modulesynopsis/name"/>

              <h3>
                <xsl:value-of select="$messages/message[@name='summary']"/>
              </h3>

			<xsl:apply-templates select="modulesynopsis/summary"/>

          <div id="quickview">

            <!-- Index of directives, automatically generated from
                 directivesynopsis/name -->
            <h3 class="directives">
              <xsl:value-of select="$messages/message[@name='directives']"/>
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
                <xsl:value-of select="$messages/message[@name='nodirectives']"/>
              </p>
            </xsl:if>

            <xsl:if test="modulesynopsis/section">
			  <h3>
                <xsl:value-of select="$messages/message[@name='topics']"/>
              </h3>
              <ul id="topics">
                <xsl:apply-templates select="modulesynopsis/section" mode="index"/>
              </ul>
            </xsl:if>

            <xsl:if test="modulesynopsis/seealso">
	      <h3>
                  <xsl:value-of select="$messages/message[@name='seealso']"/>
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
			<xsl:value-of select="$messages/message[@name='apachemodule']"/>
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