<?xml version="1.0" encoding="UTF-8"?>
<!-- $Id: apache_manualpage.xsl,v 1.8 2004/02/06 15:53:09 egr Exp $ -->

<xsl:stylesheet
	version="1.0"
	xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
>

	<xsl:import href="apache_common_elem.xsl"/>

	<xsl:template name="page_columns">
		<div class="page-content">
			<xsl:apply-templates select="manualpage"/>
		</div>
	</xsl:template>

	<xsl:template match="title">
		<h1><xsl:value-of select="."/></h1>	
	</xsl:template>


  <!--                                                            -->
  <!-- (sub)section titles                                        -->
  <!--                                                            -->

  <xsl:template match="section/title">
    <!-- Don't print the title twice -->
  </xsl:template>
  <!-- /section/title -->

	<xsl:template match="dt">
		<dt>
			<strong>
				<xsl:apply-templates/>
			</strong>
		</dt>
	</xsl:template>

</xsl:stylesheet>