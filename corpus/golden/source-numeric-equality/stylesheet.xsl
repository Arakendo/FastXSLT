<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:output omit-xml-declaration="yes"/>
  <xsl:variable name="nodes" select="/r/b"/>
  <xsl:variable name="empty" select="/r/missing"/>
  <xsl:variable name="threshold" select="2"/>
  <xsl:template match="/">
    <xsl:value-of select="$nodes = $threshold"/><xsl:text>|</xsl:text>
    <xsl:value-of select="$threshold != $nodes"/><xsl:text>|</xsl:text>
    <xsl:value-of select="$nodes = $nodes"/><xsl:text>|</xsl:text>
    <xsl:value-of select="$nodes != $nodes"/><xsl:text>|</xsl:text>
    <xsl:value-of select="$empty != $nodes"/><xsl:text>|</xsl:text>
    <xsl:variable name="spelled" select="'02'"/>
    <xsl:if test="$threshold = $spelled">typed|</xsl:if>
    <xsl:value-of select="$threshold != $spelled"/><xsl:text>|</xsl:text>
    <xsl:choose><xsl:when test="string($nodes) = string($threshold)">wrong</xsl:when><xsl:otherwise>first|</xsl:otherwise></xsl:choose>
    <xsl:if test="string($empty) = string($empty)">empty-string|</xsl:if>
    <xsl:if test="string($threshold) = $nodes">mixed-string|</xsl:if>
    <out><xsl:if test="$nodes = 2">later|</xsl:if><xsl:if test="$nodes = 2147483648">wide|</xsl:if><xsl:if test="$nodes &gt; 2">greater|</xsl:if><xsl:if test="2 &lt;= $nodes">reversed|</xsl:if><xsl:if test="$nodes &gt; $nodes">pair|</xsl:if><xsl:if test="$nodes &gt; $threshold">scalar|</xsl:if><xsl:choose><xsl:when test="$empty = 0 or $empty &gt;= 0 or $empty &gt; $nodes">wrong</xsl:when><xsl:otherwise>empty</xsl:otherwise></xsl:choose></out>
  </xsl:template>
</xsl:stylesheet>
