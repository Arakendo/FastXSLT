<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="xml" omit-xml-declaration="yes"/>
  <xsl:template match="/">
    <items><xsl:apply-templates select="catalog/item"/></items>
  </xsl:template>
  <xsl:template match="item">
    <entry><xsl:value-of select="name"/></entry>
  </xsl:template>
</xsl:stylesheet>
