#[doc = "Register `RXFA` reader"]
pub struct R(crate::R<RXFA_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<RXFA_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<RXFA_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<RXFA_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `RXFA` writer"]
pub struct W(crate::W<RXFA_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<RXFA_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for W {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<crate::W<RXFA_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<RXFA_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `FAI` reader - Rx FIFO Acknowledge Index"]
pub struct FAI_R(crate::FieldReader<u8, u8>);
impl FAI_R {
    #[inline(always)]
    pub(crate) fn new(bits: u8) -> Self {
        FAI_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FAI_R {
    type Target = crate::FieldReader<u8, u8>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FAI` writer - Rx FIFO Acknowledge Index"]
pub struct FAI_W<'a> {
    w: &'a mut W,
}
impl<'a> FAI_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x3f) | (value as u32 & 0x3f);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:5 - Rx FIFO Acknowledge Index"]
    #[inline(always)]
    pub fn fai(&self) -> FAI_R {
        FAI_R::new((self.bits & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - Rx FIFO Acknowledge Index"]
    #[inline(always)]
    pub fn fai(&mut self) -> FAI_W {
        FAI_W { w: self }
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Rx FIFO Acknowledge\n\nThis register you can [`read`](crate::reg::generic::Reg::read), [`write_with_zero`](crate::reg::generic::Reg::write_with_zero), [`reset`](crate::reg::generic::Reg::reset), [`write`](crate::reg::generic::Reg::write), [`modify`](crate::reg::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rxfa](index.html) module"]
pub struct RXFA_SPEC;
impl crate::RegisterSpec for RXFA_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [rxfa::R](R) reader structure"]
impl crate::Readable for RXFA_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [rxfa::W](W) writer structure"]
impl crate::Writable for RXFA_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets RXFA to value 0"]
impl crate::Resettable for RXFA_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
