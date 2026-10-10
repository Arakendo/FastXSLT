<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output omit-xml-declaration="yes"/>
  <xsl:strip-space elements="*"/>
  <xsl:variable name="principal" select="/"/>
  <xsl:variable name="doc" select="document('extra.xml')"/>
  <xsl:key name="which" match="value" use="'all'"/>
  <xsl:template match="/">
    <xsl:variable name="nodes" select="$doc/external/value"/>
    <xsl:variable name="alias" select="$nodes"/>
    <out>
      <xsl:for-each select="$alias">
        <xsl:sort select="@rank" data-type="number" order="descending"/>
        <item><xsl:value-of select="."/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/>:<xsl:value-of select="count(../node())"/>
          <key><xsl:value-of select="key('which','all')"/></key>
          <xsl:for-each select="$principal"><original><xsl:value-of select="external/value"/>:<xsl:value-of select="key('which','all')"/></original></xsl:for-each>
        </item>
      </xsl:for-each>
      <after><xsl:value-of select="external/value"/></after>
    </out>
  </xsl:template>
</xsl:stylesheet>
