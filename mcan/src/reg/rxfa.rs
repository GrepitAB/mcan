#[doc = "Register `RXFA` reader"]
pub type R = crate::R<RXFA_SPEC>;
#[doc = "Register `RXFA` writer"]
pub type W = crate::W<RXFA_SPEC>;
#[doc = "Field `FAI` reader - Rx FIFO Acknowledge Index"]
pub type FAI_R = crate::FieldReader;
#[doc = "Field `FAI` writer - Rx FIFO Acknowledge Index"]
pub type FAI_W<'a, REG, const O: u8> = crate::FieldWriter<'a, REG, 6, O>;
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
    #[must_use]
    pub fn fai(&mut self) -> FAI_W<RXFA_SPEC, 0> {
        FAI_W::new(self)
    }
    #[doc = r" Writes raw bits to the register."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" Passing incorrect value can cause undefined behaviour. See reference manual"]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.bits = bits;
        self
    }
}
#[doc = "Rx FIFO Acknowledge\n\nYou can [`read`](crate::reg::generic::Reg::read) this register and get [`rxfa::R`](R).  You can [`reset`](crate::reg::generic::Reg::reset), [`write`](crate::reg::generic::Reg::write), [`write_with_zero`](crate::reg::generic::Reg::write_with_zero) this register using [`rxfa::W`](W). You can also [`modify`](crate::reg::generic::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RXFA_SPEC;
impl crate::RegisterSpec for RXFA_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rxfa::R`](R) reader structure"]
impl crate::Readable for RXFA_SPEC {}
#[doc = "`write(|w| ..)` method takes [`rxfa::W`](W) writer structure"]
impl crate::Writable for RXFA_SPEC {
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets RXFA to value 0"]
impl crate::Resettable for RXFA_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
