<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output omit-xml-declaration="yes"/>
  <xsl:variable name="g"><box><item/><item/></box></xsl:variable>
  <xsl:template match="/">
    <out><xsl:for-each select="$g/box/item"><xsl:variable name="t"><box><item/><item/><item/></box></xsl:variable><v><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>:</xsl:text><xsl:for-each select="$t/box/item"><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>;</xsl:text></xsl:for-each><xsl:for-each select="$t/box/missing"><bad/></xsl:for-each></v></xsl:for-each></out>
  </xsl:template>
</xsl:stylesheet>
