<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:output omit-xml-declaration="yes"/>
  <xsl:param name="flag" select="0"/>
  <xsl:variable name="nodes" select="/r/b"/>
  <xsl:variable name="empty" select="/r/missing"/>
  <xsl:variable name="zero" select="0"/>
  <xsl:template match="/">
    <out><xsl:choose><xsl:when test="$nodes &gt; $flag">wrong</xsl:when><xsl:otherwise>equal-truth|</xsl:otherwise></xsl:choose><xsl:if test="$nodes &gt;= $flag">whole-set|</xsl:if><xsl:if test="$flag &gt; $empty">empty|</xsl:if><xsl:if test="$flag &gt; $zero">scalar</xsl:if></out>
  </xsl:template>
</xsl:stylesheet>
