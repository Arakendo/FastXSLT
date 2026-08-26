<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="xml" omit-xml-declaration="yes"/>
  <xsl:template match="/">
    <items><xsl:apply-templates/></items>
  </xsl:template>
  <xsl:template match="item">
    <entry><xsl:value-of select="."/></entry>
  </xsl:template>
</xsl:stylesheet>
