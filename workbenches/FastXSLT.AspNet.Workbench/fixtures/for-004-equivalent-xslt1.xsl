<?xml version="1.0"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:output method="xml" encoding="UTF-8" omit-xml-declaration="no"/>
  <xsl:template match="order">
    <out>
      <xsl:call-template name="total">
        <xsl:with-param name="items" select="order-item"/>
      </xsl:call-template>
    </out>
  </xsl:template>
  <xsl:template name="total">
    <xsl:param name="items"/>
    <xsl:param name="sum" select="0"/>
    <xsl:choose>
      <xsl:when test="$items">
        <xsl:call-template name="total">
          <xsl:with-param name="items" select="$items[position() &gt; 1]"/>
          <xsl:with-param name="sum" select="$sum + $items[1]/@price * $items[1]/@qty"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:otherwise>
        <xsl:value-of select="format-number($sum, '0.00')"/>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>
</xsl:stylesheet>
